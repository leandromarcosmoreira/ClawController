use crate::models::*;
use crate::db::SqlitePool;
use axum::{extract::State, Json, response::IntoResponse, http::StatusCode};
use chrono::Utc;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::RwLock;
use std::sync::Arc;
use lru::LruCache;
use dashmap::DashMap;
use metrics::{counter, histogram, gauge};
use tracing::{info, warn, error, instrument};
use std::time::Duration;

// Enhanced caching infrastructure

#[derive(Clone)]
pub struct HierarchicalCache {
    l1_cache: Arc<RwLock<LruCache<String, CachedConfig>>>,  // Memory cache
    l2_cache: Arc<RwLock<LruCache<String, CachedConfig>>>,  // Disk cache
    cache_metrics: CacheMetrics,
}

#[derive(Clone)]
struct CachedConfig {
    data: Value,
    hash: String,
    cached_at: chrono::DateTime<Utc>,
    ttl: Duration,
    access_count: u64,
    last_accessed: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetrics {
    pub l1_hits: u64,
    pub l1_misses: u64,
    pub l2_hits: u64,
    pub l2_misses: u64,
    pub evictions: u64,
    pub total_requests: u64,
}

impl HierarchicalCache {
    pub fn new(l1_capacity: usize, l2_capacity: usize) -> Self {
        Self {
            l1_cache: Arc::new(RwLock::new(LruCache::new(
                std::num::NonZeroUsize::new(l1_capacity).unwrap()
            ))),
            l2_cache: Arc::new(RwLock::new(LruCache::new(
                std::num::NonZeroUsize::new(l2_capacity).unwrap()
            ))),
            cache_metrics: CacheMetrics {
                l1_hits: 0,
                l1_misses: 0,
                l2_hits: 0,
                l2_misses: 0,
                evictions: 0,
                total_requests: 0,
            },
        }
    }

    #[instrument(skip(self))]
    pub async fn get(&self, key: &str) -> Option<Value> {
        let now = Utc::now();
        
        // Try L1 cache first
        {
            let mut l1_cache = self.l1_cache.write().await;
            self.cache_metrics.total_requests += 1;
            
            if let Some(cached) = l1_cache.get_mut(key) {
                if cached.cached_at + cached.ttl > now {
                    cached.access_count += 1;
                    cached.last_accessed = now;
                    self.cache_metrics.l1_hits += 1;
                    counter!("cache_l1_hits_total").increment(1);
                    return Some(cached.data.clone());
                } else {
                    // Expired, remove from L1
                    l1_cache.pop(key);
                }
            } else {
                self.cache_metrics.l1_misses += 1;
                counter!("cache_l1_misses_total").increment(1);
            }
        }

        // Try L2 cache
        {
            let mut l2_cache = self.l2_cache.write().await;
            if let Some(cached) = l2_cache.get_mut(key) {
                if cached.cached_at + cached.ttl > now {
                    cached.access_count += 1;
                    cached.last_accessed = now;
                    self.cache_metrics.l2_hits += 1;
                    counter!("cache_l2_hits_total").increment(1);
                    
                    // Promote to L1 cache
                    let mut l1_cache = self.l1_cache.write().await;
                    l1_cache.put(key.to_string(), cached.clone());
                    
                    return Some(cached.data.clone());
                } else {
                    // Expired, remove from L2
                    l2_cache.pop(key);
                }
            } else {
                self.cache_metrics.l2_misses += 1;
                counter!("cache_l2_misses_total").increment(1);
            }
        }

        None
    }

    #[instrument(skip(self, data))]
    pub async fn put(&self, key: String, data: Value, ttl: Duration) {
        let now = Utc::now();
        let hash = format!("{:x}", sha2::Sha256::digest(data.to_string().as_bytes()));
        
        let cached = CachedConfig {
            data: data.clone(),
            hash,
            cached_at: now,
            ttl,
            access_count: 0,
            last_accessed: now,
        };
        
        // Store in both caches
        {
            let mut l1_cache = self.l1_cache.write().await;
            l1_cache.put(key.clone(), cached.clone());
        }
        
        {
            let mut l2_cache = self.l2_cache.write().await;
            l2_cache.put(key, cached);
        }
        
        counter!("cache_puts_total").increment(1);
    }

    pub async fn get_metrics(&self) -> CacheMetrics {
        self.cache_metrics.clone()
    }

    pub async fn warm_cache_for_agents(&self, agent_ids: Vec<String>) {
        info!("Warming cache for {} agents", agent_ids.len());
        
        for agent_id in agent_ids {
            // This would typically fetch from database or external source
            // For now, we'll simulate cache warming
            let mock_config = serde_json::json!({
                "agent_id": agent_id,
                "warming": true,
                "timestamp": Utc::now()
            });
            
            self.put(
                format!("agent_config:{}", agent_id),
                mock_config,
                Duration::from_secs(3600) // 1 hour TTL
            ).await;
        }
        
        info!("Cache warming completed");
    }
}

// Resource pool management

#[derive(Clone)]
pub struct AgentPool {
    available_agents: Arc<RwLock<Vec<Agent>>>,
    busy_agents: Arc<RwLock<HashMap<String, BusyAgent>>>,
    resource_monitor: Arc<ResourceMonitor>,
    load_balancer: Arc<LoadBalancer>,
    pool_metrics: PoolMetrics,
}

#[derive(Clone)]
struct BusyAgent {
    agent: Agent,
    task_id: String,
    started_at: chrono::DateTime<Utc>,
    estimated_completion: chrono::DateTime<Utc>,
    resource_usage: ResourceUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub cpu_percent: f64,
    pub memory_mb: u64,
    pub network_io_mb: u64,
    pub disk_io_mb: u64,
    pub api_calls: u32,
    pub cost_usd: f64,
}

#[derive(Clone)]
pub struct ResourceMonitor {
    pub cpu_threshold: f64,
    pub memory_threshold_mb: u64,
    pub cost_threshold_usd: f64,
}

#[derive(Clone)]
pub struct LoadBalancer {
    pub algorithm: LoadBalancingAlgorithm,
    pub health_checker: HealthChecker,
}

#[derive(Clone)]
pub enum LoadBalancingAlgorithm {
    RoundRobin,
    LeastConnections,
    WeightedRoundRobin { weights: HashMap<String, f64> },
    PerformanceBased,
    CostOptimized,
    PredictiveBased,
}

#[derive(Clone)]
pub struct HealthChecker {
    pub check_interval: Duration,
    pub timeout: Duration,
    pub max_failures: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolMetrics {
    pub total_agents: usize,
    pub available_agents: usize,
    pub busy_agents: usize,
    pub average_wait_time: Duration,
    pub total_requests: u64,
    pub successful_allocations: u64,
    pub failed_allocations: u64,
}

impl AgentPool {
    pub fn new(agents: Vec<Agent>) -> Self {
        Self {
            available_agents: Arc::new(RwLock::new(agents)),
            busy_agents: Arc::new(RwLock::new(HashMap::new())),
            resource_monitor: Arc::new(ResourceMonitor {
                cpu_threshold: 80.0,
                memory_threshold_mb: 4096,
                cost_threshold_usd: 10.0,
            }),
            load_balancer: Arc::new(LoadBalancer {
                algorithm: LoadBalancingAlgorithm::PerformanceBased,
                health_checker: HealthChecker {
                    check_interval: Duration::from_secs(30),
                    timeout: Duration::from_secs(5),
                    max_failures: 3,
                },
            }),
            pool_metrics: PoolMetrics {
                total_agents: 0,
                available_agents: 0,
                busy_agents: 0,
                average_wait_time: Duration::from_secs(0),
                total_requests: 0,
                successful_allocations: 0,
                failed_allocations: 0,
            },
        }
    }

    #[instrument(skip(self, requirements))]
    pub async fn get_optimal_agent(&self, requirements: &TaskRequirements) -> Option<Agent> {
        let start_time = std::time::Instant::now();
        self.pool_metrics.total_requests += 1;
        
        let available_agents = self.available_agents.read().await;
        let busy_agents = self.busy_agents.read().await;
        
        // Filter agents based on requirements
        let candidates: Vec<&Agent> = available_agents
            .iter()
            .filter(|agent| self.meets_requirements(agent, requirements))
            .collect();
        
        if candidates.is_empty() {
            warn!("No available agents meet requirements");
            self.pool_metrics.failed_allocations += 1;
            return None;
        }
        
        // Select best agent based on load balancing algorithm
        let selected_agent = match &self.load_balancer.algorithm {
            LoadBalancingAlgorithm::RoundRobin => {
                candidates[self.pool_metrics.total_requests as usize % candidates.len()].clone()
            }
            LoadBalancingAlgorithm::LeastConnections => {
                candidates.iter()
                    .min_by_key(|agent| {
                        busy_agents.get(&agent.id).map(|_| 1).unwrap_or(0)
                    })
                    .copied()
            }
            LoadBalancingAlgorithm::PerformanceBased => {
                candidates.iter()
                    .max_by_key(|agent| {
                        // This would typically use performance metrics
                        // For now, use a simple heuristic
                        agent.model_failure_count
                    })
                    .copied()
            }
            LoadBalancingAlgorithm::CostOptimized => {
                candidates.iter()
                    .min_by_key(|agent| {
                        // Estimate cost based on model and configuration
                        self.estimate_agent_cost(agent)
                    })
                    .copied()
            }
            _ => candidates[0],
        };
        
        if let Some(agent) = selected_agent {
            self.pool_metrics.successful_allocations += 1;
            let wait_time = start_time.elapsed();
            self.pool_metrics.average_wait_time = 
                (self.pool_metrics.average_wait_time + wait_time) / 2;
            
            gauge!("agent_pool_available_agents").set(available_agents.len() as f64);
            gauge!("agent_pool_busy_agents").set(busy_agents.len() as f64);
            
            Some(agent.clone())
        } else {
            self.pool_metrics.failed_allocations += 1;
            None
        }
    }

    #[instrument(skip(self, agent_id, task_id))]
    pub async fn allocate_agent(&self, agent_id: &str, task_id: String) -> Result<(), String> {
        let mut available_agents = self.available_agents.write().await;
        let mut busy_agents = self.busy_agents.write().await;
        
        // Find and remove from available
        let agent_pos = available_agents.iter()
            .position(|agent| agent.id == agent_id);
        
        if let Some(pos) = agent_pos {
            let agent = available_agents.remove(pos);
            
            // Add to busy
            let busy_agent = BusyAgent {
                agent: agent.clone(),
                task_id: task_id.clone(),
                started_at: Utc::now(),
                estimated_completion: Utc::now() + chrono::Duration::minutes(30),
                resource_usage: ResourceUsage {
                    cpu_percent: 0.0,
                    memory_mb: 0,
                    network_io_mb: 0,
                    disk_io_mb: 0,
                    api_calls: 0,
                    cost_usd: 0.0,
                },
            };
            
            busy_agents.insert(agent_id.to_string(), busy_agent);
            
            self.pool_metrics.available_agents = available_agents.len();
            self.pool_metrics.busy_agents = busy_agents.len();
            
            info!("Allocated agent {} to task {}", agent_id, task_id);
            Ok(())
        } else {
            Err(format!("Agent {} not available", agent_id))
        }
    }

    #[instrument(skip(self, agent_id))]
    pub async fn release_agent(&self, agent_id: &str) -> Result<(), String> {
        let mut available_agents = self.available_agents.write().await;
        let mut busy_agents = self.busy_agents.write().await;
        
        if let Some(busy_agent) = busy_agents.remove(agent_id) {
            available_agents.push(busy_agent.agent);
            
            self.pool_metrics.available_agents = available_agents.len();
            self.pool_metrics.busy_agents = busy_agents.len();
            
            info!("Released agent {}", agent_id);
            Ok(())
        } else {
            Err(format!("Agent {} not found in busy pool", agent_id))
        }
    }

    pub async fn get_pool_metrics(&self) -> PoolMetrics {
        self.pool_metrics.clone()
    }

    fn meets_requirements(&self, agent: &Agent, requirements: &TaskRequirements) -> bool {
        // Check if agent has required skills
        if let Some(required_skills) = &requirements.required_skills {
            if let Some(agent_skills) = &agent.skills {
                if let Ok(agent_skills_vec) = serde_json::from_str::<Vec<String>>(agent_skills) {
                    for skill in required_skills {
                        if !agent_skills_vec.contains(skill) {
                            return false;
                        }
                    }
                }
            }
        }
        
        // Check security level
        if let Some(required_security) = &requirements.security_level {
            if agent.security_level < *required_security {
                return false;
            }
        }
        
        // Check resource requirements
        if let Some(max_concurrent) = agent.max_concurrent {
            if requirements.concurrency_required > max_concurrent {
                return false;
            }
        }
        
        true
    }

    fn estimate_agent_cost(&self, agent: &Agent) -> f64 {
        // Simple cost estimation based on model and configuration
        let base_cost = match agent.primary_model.as_deref() {
            Some("claude-3-sonnet") => 3.0,
            Some("gpt-4") => 4.0,
            Some("gemini-pro") => 2.0,
            _ => 1.0,
        };
        
        // Adjust for complexity
        let complexity_multiplier = match agent.thinking_default {
            Some(crate::models::ThinkingLevel::High) => 1.5,
            Some(crate::models::ThinkingLevel::XHigh) => 2.0,
            _ => 1.0,
        };
        
        base_cost * complexity_multiplier
    }
}

#[derive(Clone)]
pub struct TaskRequirements {
    pub required_skills: Option<Vec<String>>,
    pub security_level: Option<SecurityLevel>,
    pub concurrency_required: i32,
    pub estimated_duration: Duration,
    pub max_cost: Option<f64>,
    pub priority: Priority,
}

// Dynamic resource management

#[derive(Clone)]
pub struct DynamicResourceManager {
    cpu_monitor: CpuMonitor,
    memory_monitor: MemoryMonitor,
    network_monitor: NetworkMonitor,
    allocation_strategy: AllocationStrategy,
    scaling_policy: ScalingPolicy,
}

#[derive(Clone)]
pub struct CpuMonitor {
    pub current_usage: f64,
    pub threshold: f64,
    pub history: Vec<f64>,
}

#[derive(Clone)]
pub struct MemoryMonitor {
    pub current_usage_mb: u64,
    pub threshold_mb: u64,
    pub history: Vec<u64>,
}

#[derive(Clone)]
pub struct NetworkMonitor {
    pub current_io_mb: u64,
    pub threshold_mb: u64,
    pub history: Vec<u64>,
}

#[derive(Clone)]
pub enum AllocationStrategy {
    CostOptimized,
    PerformanceOptimized,
    Balanced,
    Custom(Box<dyn Fn(&ResourceRequest) -> Allocation>),
}

#[derive(Clone)]
pub struct ResourceRequest {
    pub agent_id: String,
    pub cpu_required: f64,
    pub memory_required_mb: u64,
    pub network_required_mb: u64,
    pub duration: Duration,
    pub priority: Priority,
}

#[derive(Clone)]
pub struct Allocation {
    pub agent_id: String,
    pub allocated_cpu: f64,
    pub allocated_memory_mb: u64,
    pub allocated_network_mb: u64,
    pub cost_estimate: f64,
}

#[derive(Clone)]
pub struct ScalingPolicy {
    pub min_agents: usize,
    pub max_agents: usize,
    pub scale_up_threshold: f64,
    pub scale_down_threshold: f64,
    pub cooldown_period: Duration,
    pub scale_up_increment: usize,
    pub scale_down_increment: usize,
}

impl DynamicResourceManager {
    pub fn new() -> Self {
        Self {
            cpu_monitor: CpuMonitor {
                current_usage: 0.0,
                threshold: 80.0,
                history: Vec::new(),
            },
            memory_monitor: MemoryMonitor {
                current_usage_mb: 0,
                threshold_mb: 8192,
                history: Vec::new(),
            },
            network_monitor: NetworkMonitor {
                current_io_mb: 0,
                threshold_mb: 1024,
                history: Vec::new(),
            },
            allocation_strategy: AllocationStrategy::Balanced,
            scaling_policy: ScalingPolicy {
                min_agents: 2,
                max_agents: 20,
                scale_up_threshold: 0.8,
                scale_down_threshold: 0.3,
                cooldown_period: Duration::from_secs(300), // 5 minutes
                scale_up_increment: 2,
                scale_down_increment: 1,
            },
        }
    }

    #[instrument(skip(self, request))]
    pub async fn allocate_resources(&self, request: &ResourceRequest) -> Option<Allocation> {
        // Check if resources are available
        if self.cpu_monitor.current_usage + request.cpu_required > self.cpu_monitor.threshold {
            warn!("CPU threshold exceeded for request");
            return None;
        }
        
        if self.memory_monitor.current_usage_mb + request.memory_required_mb > self.memory_monitor.threshold_mb {
            warn!("Memory threshold exceeded for request");
            return None;
        }
        
        // Calculate allocation based on strategy
        let allocation = match &self.allocation_strategy {
            AllocationStrategy::CostOptimized => self.allocate_cost_optimized(request),
            AllocationStrategy::PerformanceOptimized => self.allocate_performance_optimized(request),
            AllocationStrategy::Balanced => self.allocate_balanced(request),
            AllocationStrategy::Custom(func) => func(request),
        };
        
        if let Some(ref allocation) = allocation {
            // Update monitors
            self.cpu_monitor.current_usage += allocation.allocated_cpu;
            self.memory_monitor.current_usage_mb += allocation.allocated_memory_mb;
            self.network_monitor.current_io_mb += allocation.allocated_network_mb;
            
            counter!("resource_allocations_total").increment(1);
            gauge!("cpu_usage_percent").set(self.cpu_monitor.current_usage);
            gauge!("memory_usage_mb").set(self.cpu_monitor.current_usage as f64);
        }
        
        allocation
    }

    #[instrument(skip(self, allocation))]
    pub async fn release_resources(&self, allocation: &Allocation) {
        // Update monitors
        self.cpu_monitor.current_usage -= allocation.allocated_cpu;
        self.memory_monitor.current_usage_mb -= allocation.allocated_memory_mb;
        self.network_monitor.current_io_mb -= allocation.allocated_network_mb;
        
        counter!("resource_releases_total").increment(1);
        gauge!("cpu_usage_percent").set(self.cpu_monitor.current_usage);
        gauge!("memory_usage_mb").set(self.memory_monitor.current_usage_mb as f64);
    }

    fn allocate_cost_optimized(&self, request: &ResourceRequest) -> Option<Allocation> {
        Some(Allocation {
            agent_id: request.agent_id.clone(),
            allocated_cpu: request.cpu_required.min(50.0), // Limit CPU for cost optimization
            allocated_memory_mb: request.memory_required_mb.min(2048),
            allocated_network_mb: request.network_required_mb.min(100),
            cost_estimate: self.calculate_cost_estimate(request.cpu_required, request.memory_required_mb, 0.8),
        })
    }

    fn allocate_performance_optimized(&self, request: &ResourceRequest) -> Option<Allocation> {
        Some(Allocation {
            agent_id: request.agent_id.clone(),
            allocated_cpu: request.cpu_required,
            allocated_memory_mb: request.memory_required_mb,
            allocated_network_mb: request.network_required_mb,
            cost_estimate: self.calculate_cost_estimate(request.cpu_required, request.memory_required_mb, 1.2),
        })
    }

    fn allocate_balanced(&self, request: &ResourceRequest) -> Option<Allocation> {
        Some(Allocation {
            agent_id: request.agent_id.clone(),
            allocated_cpu: request.cpu_required * 0.8,
            allocated_memory_mb: request.memory_required_mb * 0.8,
            allocated_network_mb: request.network_required_mb * 0.8,
            cost_estimate: self.calculate_cost_estimate(request.cpu_required, request.memory_required_mb, 1.0),
        })
    }

    fn calculate_cost_estimate(&self, cpu: f64, memory_mb: u64, multiplier: f64) -> f64 {
        let base_cost = (cpu * 0.01) + (memory_mb as f64 * 0.0001);
        base_cost * multiplier
    }

    pub async fn should_scale_up(&self) -> bool {
        (self.cpu_monitor.current_usage as f64) > (self.cpu_monitor.threshold as f64 * self.scaling_policy.scale_up_threshold) ||
        (self.memory_monitor.current_usage_mb as f64) > (self.memory_monitor.threshold_mb as f64 * self.scaling_policy.scale_up_threshold)
    }

    pub async fn should_scale_down(&self) -> bool {
        (self.cpu_monitor.current_usage as f64) < (self.cpu_monitor.threshold as f64 * self.scaling_policy.scale_down_threshold) &&
        (self.memory_monitor.current_usage_mb as f64) < (self.memory_monitor.threshold_mb as f64 * self.scaling_policy.scale_down_threshold)
    }
}

// API endpoints for optimization

#[derive(Deserialize)]
pub struct CacheWarmRequest {
    pub agent_ids: Vec<String>,
    pub ttl_seconds: Option<u64>,
}

#[derive(Serialize)]
pub struct OptimizationStatus {
    pub cache_metrics: CacheMetrics,
    pub pool_metrics: PoolMetrics,
    pub resource_usage: ResourceUsage,
    pub recommendations: Vec<String>,
}

pub async fn warm_cache(
    State(app_state): State<crate::AppState>,
    Json(request): Json<CacheWarmRequest>,
) -> impl IntoResponse {
    let cache = HierarchicalCache::new(1000, 5000);
    let ttl = Duration::from_secs(request.ttl_seconds.unwrap_or(3600));
    
    cache.warm_cache_for_agents(request.agent_ids).await;
    
    Json(serde_json::json!({
        "status": "success",
        "message": "Cache warming initiated",
        "cache_metrics": cache.get_metrics().await
    }))
}

pub async fn get_optimization_status(
    State(app_state): State<crate::AppState>,
) -> impl IntoResponse {
    // This would typically use actual instances from app state
    let cache = HierarchicalCache::new(1000, 5000);
    let agent_pool = AgentPool::new(vec![]);
    let resource_manager = DynamicResourceManager::new();
    
    let status = OptimizationStatus {
        cache_metrics: cache.get_metrics().await,
        pool_metrics: agent_pool.get_pool_metrics().await,
        resource_usage: ResourceUsage {
            cpu_percent: resource_manager.cpu_monitor.current_usage,
            memory_mb: resource_manager.memory_monitor.current_usage_mb,
            network_io_mb: resource_manager.network_monitor.current_io_mb,
            disk_io_mb: resource_manager.disk_monitor.current_io_mb,
            api_calls: 0,
            cost_usd: 0.0,
        },
        recommendations: generate_recommendations(&resource_manager).await,
    };
    
    Json(status)
}

async fn generate_recommendations(resource_manager: &DynamicResourceManager) -> Vec<String> {
    let mut recommendations = Vec::new();
    
    if resource_manager.should_scale_up().await {
        recommendations.push("Consider scaling up - resource usage is high".to_string());
    }
    
    if resource_manager.should_scale_down().await {
        recommendations.push("Consider scaling down - resource usage is low".to_string());
    }
    
    if resource_manager.cpu_monitor.current_usage > 70.0 {
        recommendations.push("CPU usage is high - consider optimizing agent configurations".to_string());
    }
    
    if resource_manager.memory_monitor.current_usage_mb > 6000 {
        recommendations.push("Memory usage is high - consider implementing memory optimization".to_string());
    }
    
    recommendations
}
pub async fn get_pool_status() -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "Not implemented")
}

pub async fn get_resource_status() -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "Not implemented")
}
