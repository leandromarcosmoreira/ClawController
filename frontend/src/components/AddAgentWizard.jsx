import { useState, useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { useMissionStore } from '../store/useMissionStore'

const STEPS = {
  DESCRIBE: 1,
  LOADING: 2,
  REVIEW: 3,
}

// Pre-configured orchestrator template
const ORCHESTRATOR_CONFIG = {
  id: 'main',
  name: 'Orquestrador',
  emoji: '🎯',
  model: '',
  soul: `# Agente Orquestrador

Você é o orquestrador principal e líder da equipe. Seu papel é:

1. **Receber tarefas** do humano e dividi-las em trabalho acionável
2. **Delegar** a agentes especialistas com base em suas capacidades
3. **Monitorar progresso** e garantir que as tarefas sejam concluídas adequadamente
4. **Reportar** com resumos e resultados

## Estilo de Trabalho

- Ser proativo esclarecendo requisitos antes de delegar
- Verificar tarefas delegadas e acompanhar se necessário
- Sintetizar resultados de múltiplos agentes em respostas coerentes
- Escalar bloqueios ou decisões que precisam de entrada humana

## Comunicação

- Manter o humano informado do progresso significativo
- Ser conciso mas completo nas atualizações de status
- Sinalizar riscos ou preocupações cedo

Você é o coordenador central. Outros agentes reportam a você.`,
  tools: `# Ferramentas e Integrações

## Ferramentas Disponíveis

Documente quaisquer configurações de ferramentas, chaves de API ou integrações aqui.

## Equipe de Agentes

Liste seus agentes especialistas e suas capacidades:

- **Nome do Agente**: Descrição do que faz

## Preferências

- Estilo de comunicação preferido
- Horário de trabalho
- Quaisquer instruções especiais`,
  agentsMd: `# Espaço de Trabalho

Este é o espaço de trabalho principal do orquestrador.

## Memória

Use arquivos \`memory/YYYY-MM-DD.md\` para rastrear trabalho diário e decisões.

## Diretrizes

- Delegue tarefas complexas a especialistas
- Mantenha o humano informado do progresso
- Documente decisões importantes`,
}

export default function AddAgentWizard({ mode }) {
  const { t } = useTranslation()
  const availableModels = useMissionStore((s) => s.availableModels)
  const loadingAgentManagement = useMissionStore((s) => s.loadingAgentManagement)
  const closeAddAgentWizard = useMissionStore((s) => s.closeAddAgentWizard)
  const generateAgentConfig = useMissionStore((s) => s.generateAgentConfig)
  const createAgent = useMissionStore((s) => s.createAgent)

  // Determine initial step based on mode
  const initialStep = mode === 'orchestrator' ? STEPS.REVIEW : STEPS.DESCRIBE

  const [step, setStep] = useState(initialStep)
  const [description, setDescription] = useState('')
  const [originalDescription, setOriginalDescription] = useState('')

  // Agent config fields
  const [agentId, setAgentId] = useState('')
  const [agentName, setAgentName] = useState('')
  const [agentEmoji, setAgentEmoji] = useState('🤖')
  const [agentModel, setAgentModel] = useState('')
  const [soulMd, setSoulMd] = useState('')
  const [toolsMd, setToolsMd] = useState('')
  const [agentsMd, setAgentsMd] = useState('')

  const [error, setError] = useState('')

  // Initialize with orchestrator config if in orchestrator mode
  useEffect(() => {
    if (mode === 'orchestrator') {
      setAgentId(ORCHESTRATOR_CONFIG.id)
      setAgentName(ORCHESTRATOR_CONFIG.name)
      setAgentEmoji(ORCHESTRATOR_CONFIG.emoji)
      setAgentModel(ORCHESTRATOR_CONFIG.model)
      setSoulMd(ORCHESTRATOR_CONFIG.soul)
      setToolsMd(ORCHESTRATOR_CONFIG.tools)
      setAgentsMd(ORCHESTRATOR_CONFIG.agentsMd)
    }
  }, [mode])

  const handleGenerate = async () => {
    if (!description.trim()) {
      setError('Por favor, descreva o que o agente deve fazer')
      return
    }

    setError('')
    setOriginalDescription(description)
    setStep(STEPS.LOADING)

    try {
      const config = await generateAgentConfig(description)
      setAgentId(config.id || '')
      setAgentName(config.name || '')
      setAgentEmoji(config.emoji || '🤖')
      setAgentModel(config.model || '')
      setSoulMd(config.soul || '')
      setToolsMd(config.tools || '')
      setAgentsMd(config.agentsMd || '')
      setStep(STEPS.REVIEW)
    } catch (err) {
      setError(t('agent_management.generate_config_error'))
      setStep(STEPS.DESCRIBE)
    }
  }

  const handleRefine = () => {
    setDescription(originalDescription + '\n\n[Refinamento]: ')
    setStep(STEPS.DESCRIBE)
  }

  const handleCreate = async () => {
    if (!agentId.trim()) {
      setError('ID do Agente é obrigatório')
      return
    }

    if (!/^[a-z0-9-]+$/.test(agentId)) {
      setError('ID do Agente pode conter apenas letras minúsculas, números e hífens')
      return
    }

    setError('')

    try {
      await createAgent({
        id: agentId,
        name: agentName,
        emoji: agentEmoji,
        model: agentModel,
        soul: soulMd,
        tools: toolsMd,
        agentsMd: agentsMd,
      })
    } catch (err) {
      setError(err.message || 'Falha ao criar agente')
    }
  }

  const handleIdChange = (e) => {
    setAgentId(e.target.value.toLowerCase().replace(/[^a-z0-9-]/g, '-'))
  }

  const isOrchestrator = mode === 'orchestrator'
  const title = isOrchestrator ? '🎯 Inicializar Orquestrador' : '✨ Criar Novo Agente'
  const stepLabel = step === STEPS.DESCRIBE ? 'Passo 1 de 2' :
    step === STEPS.LOADING ? 'Gerando...' :
      isOrchestrator ? 'Revisar & Criar' : 'Passo 2 de 2'

  return (
    <div className="modal-overlay agent-wizard-overlay" onClick={closeAddAgentWizard}>
      <div className="modal add-agent-wizard" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <div>
            <span className="modal-label">{stepLabel}</span>
            <h2>{title}</h2>
          </div>
          <button className="icon-button" onClick={closeAddAgentWizard}>
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <path d="M18 6L6 18M6 6l12 12" />
            </svg>
          </button>
        </div>

        <div className="modal-content">
          {error && (
            <div className="wizard-error">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <circle cx="12" cy="12" r="10" />
                <path d="M12 8v4M12 16h.01" />
              </svg>
              {error}
            </div>
          )}

          {step === STEPS.DESCRIBE && (
            <div className="wizard-step">
              <p className="wizard-instruction">
                {t('agent_management.describe_agent_instruction')}
              </p>

              <div className="field">
                <textarea
                  value={description}
                  onChange={(e) => setDescription(e.target.value)}
                  placeholder={t('agent_management.description_placeholder')}
                  rows={8}
                  className="wizard-textarea"
                  autoFocus
                />
              </div>

              <div className="wizard-examples">
                <span>{t('agent_management.examples')}</span>
                <button onClick={() => setDescription(t('agent_management.example_backend'))}>
                  {t('agent_management.backend_dev')}
                </button>
                <button onClick={() => setDescription(t('agent_management.example_sales'))}>
                  {t('agent_management.sales_agent')}
                </button>
                <button onClick={() => setDescription(t('agent_management.example_researcher'))}>
                  {t('agent_management.researcher')}
                </button>
              </div>
            </div>
          )}

          {step === STEPS.LOADING && (
            <div className="wizard-loading">
              <div className="loading-spinner large" />
              <p>{t('agent_management.generating_config')}</p>
            </div>
          )}

          {step === STEPS.REVIEW && (
            <div className="wizard-step wizard-review">
              {isOrchestrator && (
                <p className="wizard-instruction orchestrator-intro">
                  {t('agent_management.orchestrator_intro')}
                </p>
              )}

              <div className="wizard-review-row">
                <div className="field" style={{ flex: 1 }}>
                  <label>{t('agent_management.agent_id')}</label>
                  <input
                    type="text"
                    value={agentId}
                    onChange={handleIdChange}
                    placeholder={t('agent_management.agent_id_placeholder')}
                  />
                  <span className="field-hint">{t('agent_management.agent_id_hint')}</span>
                </div>
                <div className="field" style={{ width: '80px' }}>
                  <label>{t('agent_management.emoji')}</label>
                  <input
                    type="text"
                    value={agentEmoji}
                    onChange={(e) => setAgentEmoji(e.target.value)}
                    placeholder={t('agent_management.emoji_placeholder')}
                  />
                </div>
              </div>

              <div className="wizard-review-row">
                <div className="field" style={{ flex: 1 }}>
                  <label>{t('agent_management.name')}</label>
                  <input
                    type="text"
                    value={agentName}
                    onChange={(e) => setAgentName(e.target.value)}
                    placeholder={t('agent_management.name_placeholder')}
                  />
                </div>
                <div className="field" style={{ flex: 1 }}>
                  <label>{t('agent_management.model')}</label>
                  <select
                    value={agentModel}
                    onChange={(e) => setAgentModel(e.target.value)}
                    className="wizard-select"
                  >
                    <option value="">{t('agent_management.select_model')}</option>
                    {availableModels && availableModels.length > 0 ? availableModels.map((model) => (
                      <option key={model.id} value={model.id}>
                        {model.id}
                      </option>
                    )) : (
                      <option value="" disabled>{t('agent_management.no_models_available')}</option>
                    )}
                  </select>
                </div>
              </div>

              <div className="field">
                <label>{t('agent_management.soul_md')}</label>
                <textarea
                  value={soulMd}
                  onChange={(e) => setSoulMd(e.target.value)}
                  rows={10}
                  className="wizard-textarea wizard-textarea--code"
                />
              </div>

              <div className="field">
                <label>{t('agent_management.tools_md')}</label>
                <textarea
                  value={toolsMd}
                  onChange={(e) => setToolsMd(e.target.value)}
                  rows={6}
                  className="wizard-textarea wizard-textarea--code"
                />
              </div>
            </div>
          )}
        </div>

        <div className="modal-actions">
          {step === STEPS.DESCRIBE && (
            <>
              <button className="secondary-button" onClick={closeAddAgentWizard}>
                {t('agent_management.cancel')}
              </button>
              <button
                className="primary-button"
                onClick={handleGenerate}
                disabled={!description.trim()}
              >
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <path d="M12 2l3.09 6.26L22 9.27l-5 4.87L18.18 22 12 18.27 5.82 22 7 14.14l-5-4.87 6.91-1.01L12 2z" />
                </svg>
                {t('agent_management.generate_config')}
              </button>
            </>
          )}

          {step === STEPS.REVIEW && (
            <>
              {!isOrchestrator && (
                <button className="secondary-button" onClick={handleRefine}>
                  {t('agent_management.refine')}
                </button>
              )}
              <div style={{ flex: 1 }} />
              <button className="secondary-button" onClick={closeAddAgentWizard}>
                {t('agent_management.cancel')}
              </button>
              <button
                className="primary-button"
                onClick={handleCreate}
                disabled={loadingAgentManagement || !agentId.trim()}
              >
                {loadingAgentManagement ? t('agent_management.creating') : t('agent_management.create_agent')}
              </button>
            </>
          )}
        </div>
      </div>
    </div>
  )
}
