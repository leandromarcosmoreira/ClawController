> **⚠️ FORK** — Este repositório é um fork de [mdonan90/ClawController](https://github.com/mdonan90/ClawController).
> Repositório deste fork: [leandromarcosmoreira/ClawController](https://github.com/leandromarcosmoreira/ClawController)

---

# ClawController

**Central de Controle para Agentes [OpenClaw](https://openclaw.ai)**

Mantenha seus agentes de IA organizados e responsáveis. O ClawController oferece visibilidade sobre o que seus agentes OpenClaw estão fazendo, atribui trabalho estruturado e acompanha o progresso — para que você não fique apenas torcendo para que estejam na tarefa certa.

**O problema:** Você tem vários agentes OpenClaw rodando, mas como sabe o que eles estão realmente fazendo? Estão travados? Terminaram? O que vem a seguir?

**A solução:** O ClawController fornece um painel visual onde você pode:
- Ver todos os seus agentes e o status atual de relance
- Atribuir tarefas estruturadas com entregas claras
- Acompanhar o progresso por um fluxo de trabalho definido
- Rotear trabalho para o agente certo automaticamente
- Revisar o trabalho concluído antes de fechar tarefas

---

## Índice

- [Funcionalidades](#funcionalidades)
- [Capturas de Tela](#capturas-de-tela)
- [Início Rápido](#início-rápido)
- [Arquitetura](#arquitetura)
- [Configuração](#configuração)
- [Criando Agentes](#criando-agentes)
- [Fluxo de Tarefas](#fluxo-de-tarefas)
- [Regras de Atribuição Automática](#regras-de-atribuição-automática)
- [Tarefas Recorrentes](#tarefas-recorrentes)
- [Referência da API](#referência-da-api)
- [Integração com OpenClaw](#integração-com-openclaw)
- [Personalização](#personalização)
- [Contribuindo](#contribuindo)

---

## Por que o ClawController?

Executar vários agentes OpenClaw é poderoso, mas pode ficar caótico:
- Agentes trabalham em sessões isoladas — você perde o controle de quem está fazendo o quê
- Não há um lugar central para ver o progresso de todos os agentes
- O trabalho fica duplicado ou esquecido
- Difícil revisar o resultado antes de entregar

O ClawController resolve isso dando a você **um único lugar** para gerenciar o trabalho, não os agentes em si. O OpenClaw cuida da IA. O ClawController cuida do fluxo de trabalho.

## Funcionalidades

| Funcionalidade | Descrição |
|----------------|-----------|
| **Status do Agente** | Veja quais agentes OpenClaw estão online, trabalhando ou ociosos |
| **Quadro Kanban** | Arraste e solte tarefas por CAIXA DE ENTRADA → ATRIBUÍDA → EM ANDAMENTO → REVISÃO → CONCLUÍDA |
| **Atribuição de Tarefas** | Atribua trabalho a agentes específicos com descrições e datas de entrega |
| **Log de Atividades** | Agentes relatam progresso; você vê em tempo real |
| **Atribuição Automática** | Roteie tarefas para agentes automaticamente com base em tags |
| **Portão de Revisão** | O trabalho vai para REVISÃO antes de CONCLUÍDA — nada é entregue sem aprovação |
| **Chat da Equipe** | @mencione agentes para enviar mensagens diretamente |
| **Tarefas Recorrentes** | Agende trabalho repetitivo em cronogramas cron |
| **Atualizações via WebSocket** | O painel atualiza em tempo real conforme os agentes trabalham |

---

## Capturas de Tela

### Painel de Operações SaaS
![Painel SaaS](screenshots/saas-dashboard.png)
*Gerencie sua equipe de IA com quadros kanban, monitoramento de status dos agentes e feeds de atividade em tempo real.*

### Operações de Trading
![Painel de Trading](screenshots/trading-dashboard.png)
*Coordene agentes de trading com fluxos de trabalho especializados e gerenciamento de tarefas focado no mercado.*

### Fluxo de Agência
![Painel de Agência](screenshots/agency-dashboard.png)
*Gerencie uma agência criativa com agentes de redação, design e especialistas trabalhando em paralelo.*

---

## Início Rápido

### Pré-requisitos

- **Node.js 18+** (para o frontend)
- **Python 3.10+** (para o backend)

### Instalação

```bash
# Clonar o repositório (fork)
git clone git@github.com:leandromarcosmoreira/ClawController.git
cd ClawController

# Configuração do backend
cd backend
python -m venv venv
source venv/bin/activate  # Windows: venv\Scripts\activate
pip install -r requirements.txt

# Configuração do frontend
cd ../frontend
pnpm install
```

### Executando

**Opção 1: Usar o script de inicialização**
```bash
./start.sh
```

**Opção 2: Inicialização manual**
```bash
# Terminal 1 - Backend
cd backend
source venv/bin/activate
uvicorn main:app --host 0.0.0.0 --port 8000 --reload

# Terminal 2 - Frontend
cd frontend
pnpm dev -- --port 5001 --host 0.0.0.0
```

**Acesse o painel:** http://localhost:5001

### Parando
```bash
./stop.sh
```

---

## Seu Primeiro Agente

Com o painel rodando, crie seu primeiro agente:

```bash
# Criar um agente desenvolvedor simples
curl -X POST http://localhost:8000/api/agents \
  -H "Content-Type: application/json" \
  -d '{
    "id": "dev",
    "name": "Agente Dev",
    "role": "developer",
    "description": "Lida com tarefas de programação e trabalho técnico",
    "avatar": "💻",
    "status": "idle"
  }'
```

**Verificação:** Atualize o painel em http://localhost:5001 e você deverá ver "Agente Dev 💻" na barra lateral.

**Próximos Passos:** Veja [Criando Agentes](#criando-agentes) para criação assistida por IA e configuração avançada.

---

## Arquitetura

```
ClawController/
├── backend/
│   ├── main.py          # Aplicação FastAPI + todos os endpoints
│   ├── models.py        # Modelos SQLAlchemy (Tarefa, Agente, etc.)
│   ├── database.py      # Configuração da conexão com o banco de dados
│   └── requirements.txt # Dependências Python
├── frontend/
│   ├── src/
│   │   ├── App.jsx      # Componente React principal
│   │   ├── components/  # Componentes de UI (Cabeçalho, Kanban, etc.)
│   │   └── store/       # Gerenciamento de estado com Zustand
│   └── package.json     # Dependências Node
├── start.sh             # Inicia ambos os serviços
└── stop.sh              # Para ambos os serviços
```

### Stack Tecnológica

- **Frontend:** React 18 + Vite + Tailwind CSS + Zustand
- **Backend:** FastAPI + SQLite + SQLAlchemy
- **Tempo Real:** WebSockets para atualizações ao vivo

---

## Configuração

### Variáveis de Ambiente

Crie um arquivo `.env` no diretório backend (opcional):

```env
# Caminho do banco de dados (padrão: ./data/mission_control.db)
DATABASE_URL=sqlite:///./data/mission_control.db

# Caminho de configuração do OpenClaw para status ao vivo dos agentes
OPENCLAW_CONFIG_PATH=~/.openclaw/config.yaml
```

### Configuração do Frontend

Edite `frontend/src/App.jsx` para alterar a URL da API:

```javascript
const API_BASE = 'http://localhost:8000/api';
```

Para produção, aponte para a URL do seu backend.

---

## Criando Agentes

### Criação Assistida por IA (Recomendado)

O ClawController pode gerar configurações de agentes a partir de descrições em linguagem natural:

**Passo 1: Descreva seu Agente**

![Criação de Agente Passo 1](screenshots/agent-create-step1.png)

1. Clique em **+ Novo Agente**
2. Descreva o que você quer: *"Um analista de pesquisa de mercado que entende macro de longo prazo enquanto fornece orientação micro"*
3. Ou clique em um template: `Dev Backend`, `Agente de Vendas`, `Pesquisador`
4. Clique em **Gerar Configuração**

**Passo 2: Revisar e Personalizar**

![Criação de Agente Passo 2](screenshots/agent-create-step2.png)

O sistema gera:
- **ID e Nome do Agente** — sugeridos automaticamente com base na sua descrição
- **Emoji** — identificador visual
- **Modelo** — modelo recomendado (Sonnet, Opus, Haiku, etc.)
- **SOUL.md** — personalidade, competências e diretrizes de comportamento
- **TOOLS.md** — ferramentas disponíveis e integrações

Você pode editar qualquer campo, refinar o SOUL.md ou clicar em **← Refinar** para ajustar sua descrição. Quando estiver pronto, clique em **Criar Agente**.

### Criação Manual (API)

**Exemplo Completo - Agente Líder:**

```bash
curl -X POST http://localhost:8000/api/agents \
  -H "Content-Type: application/json" \
  -d '{
    "id": "main",
    "name": "Líder do Projeto",
    "role": "LEAD",
    "description": "Orquestrador principal e revisor de tarefas",
    "avatar": "👤",
    "status": "STANDBY",
    "workspace": "/home/usuario/projetos"
  }'
```

**Resposta Esperada:**
```json
{
  "id": "main",
  "name": "Líder do Projeto",
  "role": "LEAD",
  "description": "Orquestrador principal e revisor de tarefas",
  "avatar": "👤",
  "status": "STANDBY"
}
```

**Importante:** Defina exatamente **um** agente com `"role": "LEAD"` — este agente irá:
- Receber notificações de conclusão de tarefas
- Ser o revisor padrão para tarefas em status REVISÃO
- Coordenar o trabalho entre sua equipe de agentes

**Agente Desenvolvedor Simples:**
```bash
curl -X POST http://localhost:8000/api/agents \
  -H "Content-Type: application/json" \
  -d '{
    "id": "dev",
    "name": "Agente Dev",
    "role": "INT",
    "avatar": "💻",
    "status": "IDLE"
  }'
```

### Papéis dos Agentes

| Papel | Emblema | Uso Típico |
|-------|---------|------------|
| `LEAD` | Líder | Agente orquestrador que delega para outros, revisa tarefas |
| `INT` | Int | Agentes de integração - desenvolvedores, analistas, trabalhadores gerais |
| `SPC` | Spc | Especialistas - experts de domínio (trading, design, jurídico, etc.) |

**Diretrizes de Papéis:**
- **Um LEAD obrigatório** — lida com revisões de tarefas e coordenação da equipe
- **Múltiplos agentes INT** — sua força de trabalho principal para a maioria das tarefas
- **Agentes SPC** — especialistas para trabalho específico de domínio

### Status dos Agentes

| Status | Indicador | Significado |
|--------|-----------|-------------|
| `WORKING` | 🟢 Verde (pulsando) | Processando uma tarefa atualmente |
| `IDLE` | 🟡 Amarelo | Disponível, aguardando trabalho |
| `STANDBY` | ⚫ Cinza | Configurado mas inativo - pronto para ativar |
| `OFFLINE` | 🔴 Vermelho | Não configurado ou inacessível |

**Atualizações de Status:** O status do agente é detectado automaticamente a partir da atividade de sessão do OpenClaw e das atribuições de tarefas.

---

## Fluxo de Tarefas

### Ciclo de Vida da Tarefa

```
CAIXA DE ENTRADA → ATRIBUÍDA → EM ANDAMENTO → REVISÃO → CONCLUÍDA
```

| Status | Descrição | Gatilho |
|--------|-----------|---------|
| **CAIXA DE ENTRADA** | Não atribuída, precisa de triagem | Padrão para novas tarefas |
| **ATRIBUÍDA** | Atribuída ao agente, não iniciada | Atribuição manual ou automática |
| **EM ANDAMENTO** | Agente trabalhando ativamente | Primeira entrada no log de atividades |
| **REVISÃO** | Trabalho concluído, precisa de aprovação | Agente diz "concluído/feito/finalizado" |
| **CONCLUÍDA** | Aprovada e fechada | Aprovação manual apenas |

### Criando Tarefas

Tarefas podem ser criadas a partir de múltiplas superfícies:

- **Painel:** Clique no botão **+ Nova Tarefa**
- **Discord:** Envie uma mensagem ao seu agente OpenClaw com uma descrição de tarefa
- **Telegram:** Envie tarefas via seu bot Telegram conectado
- **Chat da Equipe:** Use o chat integrado para criar e atribuir tarefas

**Via API:**
```bash
curl -X POST http://localhost:8000/api/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Construir página de login",
    "description": "Criar um formulário de login responsivo com suporte a OAuth",
    "priority": "high",
    "tags": ["coding", "frontend"],
    "assignee_id": "dev"
  }'
```

### Campos da Tarefa

| Campo | Tipo | Descrição |
|-------|------|-----------|
| `title` | string | Título da tarefa (obrigatório) |
| `description` | string | Descrição detalhada |
| `priority` | enum | `low`, `medium`, `high`, `urgent` |
| `tags` | array | Rótulos para categorização |
| `assignee_id` | string | ID do agente para atribuir |
| `due_date` | datetime | Prazo opcional |
| `status` | enum | Status atual |

### Registrando Atividade

Agentes devem registrar seu progresso:

```bash
curl -X POST http://localhost:8000/api/tasks/{task_id}/activity \
  -H "Content-Type: application/json" \
  -d '{
    "agent_id": "dev",
    "message": "Comecei a trabalhar no layout do formulário de login"
  }'
```

Palavras-chave de atividade que acionam mudanças de status:
- **→ EM ANDAMENTO:** Qualquer atividade em uma tarefa ATRIBUÍDA
- **→ REVISÃO:** "concluído", "feito", "finalizado", "pronto para revisão"

---

## Regras de Atribuição Automática

Configure o roteamento automático de tarefas com base em tags.

### Configuração

Edite `backend/main.py`:

```python
# Regras de atribuição automática: tag -> agent_id
ASSIGNMENT_RULES = {
    "coding": "dev",
    "frontend": "dev",
    "backend": "dev",
    "trading": "trader",
    "analysis": "analyst",
    "marketing": "brand",
    "writing": "writer",
    "design": "designer",
    "support": "support",
}
```

### Como Funciona

1. Quando uma tarefa é criada com tags, o sistema verifica cada tag contra as regras
2. A primeira regra correspondente vence
3. A tarefa é automaticamente atribuída a esse agente
4. O status muda de CAIXA DE ENTRADA para ATRIBUÍDA

### Exemplo

```bash
# Esta tarefa será auto-atribuída a "dev" por causa da tag "coding"
curl -X POST http://localhost:8000/api/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Corrigir bug de autenticação",
    "tags": ["coding", "urgent"]
  }'
```

---

## Tarefas Recorrentes

Agende tarefas que se repetem em um cronograma.

### Criando Tarefas Recorrentes

**Via UI:** Painel de Tarefas → Aba Tarefas Recorrentes → + Nova Tarefa Recorrente

**Via API:**
```bash
curl -X POST http://localhost:8000/api/recurring-tasks \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Resumo diário de standup",
    "description": "Compilar e publicar relatório de progresso diário",
    "schedule": "0 9 * * 1-5",
    "assignee_id": "lead",
    "tags": ["daily", "reporting"],
    "enabled": true
  }'
```

### Formato de Cronograma (Cron)

```
┌───────────── minuto (0-59)
│ ┌───────────── hora (0-23)
│ │ ┌───────────── dia do mês (1-31)
│ │ │ ┌───────────── mês (1-12)
│ │ │ │ ┌───────────── dia da semana (0-6, Dom=0)
│ │ │ │ │
* * * * *
```

**Exemplos:**
- `0 9 * * 1-5` — 9h, segunda a sexta
- `0 */2 * * *` — A cada 2 horas
- `0 0 1 * *` — Primeiro dia de cada mês à meia-noite

### Gerenciando Tarefas Recorrentes

- **Pausar:** `PATCH /api/recurring-tasks/{id}` com `{"enabled": false}`
- **Ver execuções:** `GET /api/recurring-tasks/{id}/runs`
- **Excluir:** `DELETE /api/recurring-tasks/{id}`

---

## Referência da API

### Tarefas

| Método | Endpoint | Descrição |
|--------|----------|-----------|
| `GET` | `/api/tasks` | Listar todas as tarefas |
| `POST` | `/api/tasks` | Criar tarefa |
| `GET` | `/api/tasks/{id}` | Obter tarefa |
| `PATCH` | `/api/tasks/{id}` | Atualizar tarefa |
| `DELETE` | `/api/tasks/{id}` | Excluir tarefa |
| `POST` | `/api/tasks/{id}/activity` | Registrar atividade |
| `GET` | `/api/tasks/{id}/activity` | Obter atividade |

### Agentes

| Método | Endpoint | Descrição |
|--------|----------|-----------|
| `GET` | `/api/agents` | Listar todos os agentes |
| `POST` | `/api/agents` | Criar agente |
| `PATCH` | `/api/agents/{id}` | Atualizar agente |
| `DELETE` | `/api/agents/{id}` | Excluir agente |

### Chat

| Método | Endpoint | Descrição |
|--------|----------|-----------|
| `GET` | `/api/chat` | Obter mensagens |
| `POST` | `/api/chat` | Enviar mensagem |
| `POST` | `/api/chat/send-to-agent` | Rotear para agente |

### Tarefas Recorrentes

| Método | Endpoint | Descrição |
|--------|----------|-----------|
| `GET` | `/api/recurring-tasks` | Listar todas |
| `POST` | `/api/recurring-tasks` | Criar |
| `PATCH` | `/api/recurring-tasks/{id}` | Atualizar |
| `DELETE` | `/api/recurring-tasks/{id}` | Excluir |
| `GET` | `/api/recurring-tasks/{id}/runs` | Histórico de execuções |

### WebSocket

Conecte-se a `ws://localhost:8000/ws` para atualizações em tempo real:

```javascript
const ws = new WebSocket('ws://localhost:8000/ws');
ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  // Tratar: task_created, task_updated, agent_status, chat_message, etc.
};
```

---

## Integração com OpenClaw

O ClawController foi construído para o [OpenClaw](https://openclaw.ai). Veja como eles se conectam:

### Status ao Vivo dos Agentes

O ClawController lê sua configuração do OpenClaw para mostrar o status real dos agentes:

```python
# Em backend/main.py
OPENCLAW_CONFIG_PATH = os.path.expanduser("~/.openclaw/config.yaml")
```

Agentes definidos na sua configuração do OpenClaw aparecem automaticamente com indicadores de status ao vivo.

### Roteando Mensagens para Agentes

Quando você @menciona um agente no Chat da Equipe, o ClawController roteia a mensagem via:
```bash
openclaw agent --agent {agent_id} --message "{sua mensagem}"
```

Isso acorda o agente em sua própria sessão e entrega sua mensagem.

### Configurando Seus Agentes

**Importante:** Seus agentes precisam de instruções para usar o ClawController corretamente. Adicione o seguinte ao `TOOLS.md` ou `AGENTS.md` de cada agente:

```markdown
## Integração com ClawController

**Base da API:** `http://localhost:8000/api`

### Quando receber uma tarefa:
1. Verificar tarefas: `GET /api/tasks?assignee_id={seu_id}&status=ASSIGNED`
2. Registrar progresso enquanto trabalha (a cada etapa significativa)
3. Quando terminar, postar atividade com "concluído" ou "feito"
4. Aguardar aprovação humana

### Registrando Atividade (OBRIGATÓRIO enquanto trabalha)
curl -X POST http://localhost:8000/api/tasks/{TASK_ID}/activity \
  -H "Content-Type: application/json" \
  -d '{"agent_id": "SEU_AGENT_ID", "message": "O que você fez"}'

### Ciclo de Vida da Tarefa
- ATRIBUÍDA → Tarefa dada a você
- EM ANDAMENTO → Acionado automaticamente no primeiro log de atividade
- REVISÃO → Diga "concluído" na atividade para acionar
- CONCLUÍDA → Humano aprova (nunca defina isso você mesmo)

### Regras Principais
- Sempre registre atividade — o progresso é rastreado via logs de atividade
- Não pule REVISÃO — humanos aprovam antes de CONCLUÍDA
- Use atualizações descritivas — ajuda os humanos a entender o progresso
```

Um template completo está disponível em `AGENT_INSTRUCTIONS.md` no repositório.

---

## Personalização

### Temas

O tema "Cyber Claw" usa Tailwind CSS. Edite `frontend/tailwind.config.js`:

```javascript
module.exports = {
  theme: {
    extend: {
      colors: {
        primary: '#F97316',      // Destaque laranja
        background: '#09090B',   // Quase preto
        surface: '#18181B',      // Fundos de cartão
      }
    }
  }
}
```

### Adicionando Status de Tarefas

Edite `backend/models.py`:

```python
class TaskStatus(str, Enum):
    INBOX = "INBOX"
    ASSIGNED = "ASSIGNED"
    IN_PROGRESS = "IN_PROGRESS"
    BLOCKED = "BLOCKED"  # Adicionar novo status
    REVIEW = "REVIEW"
    DONE = "DONE"
```

Em seguida, atualize as colunas do kanban no frontend em `App.jsx`.

### Papéis de Agentes Personalizados

Edite `backend/models.py`:

```python
class AgentRole(str, Enum):
    LEAD = "lead"
    DEVELOPER = "developer"
    ANALYST = "analyst"
    SPECIALIST = "specialist"
    SUPPORT = "support"
    CREATIVE = "creative"  # Adicionar novo papel
```

### Adicionando Novos Endpoints de API

Adicione a `backend/main.py`:

```python
@app.get("/api/endpoint-personalizado")
def endpoint_personalizado(db: Session = Depends(get_db)):
    # Sua lógica aqui
    return {"status": "ok"}
```

---

## Implantação

### Build de Produção

```bash
# Build do frontend
cd frontend
npm run build

# Sirva com nginx ou copie dist/ para seu host estático
```

### Recomendações de Ambiente

- **Backend:** Execute com gunicorn + workers uvicorn
- **Frontend:** Sirva de CDN ou nginx
- **Banco de Dados:** SQLite funciona para equipes pequenas; PostgreSQL para escala

---

## Solução de Problemas

### Porta Já em Uso

**Problema:** `Error: listen EADDRINUSE: address already in use :::8000` ou `:::5001`

**Solução:**
```bash
# Encontrar processos usando as portas
lsof -i :8000  # Porta do backend
lsof -i :5001  # Porta do frontend

# Matar processos se necessário
kill -9 <PID>

# Ou usar portas diferentes
uvicorn main:app --port 8001  # Backend
npm run dev -- --port 5002   # Frontend
```

### Problemas de CORS com Acesso Remoto

**Problema:** O painel mostra "Falha na Conexão" ao acessar remotamente

**Solução:**
```bash
# Backend: Permitir todas as origens (apenas desenvolvimento)
uvicorn main:app --host 0.0.0.0 --port 8000

# Frontend: Habilitar acesso à rede
npm run dev -- --host 0.0.0.0 --port 5001

# Acesse via: http://SEU_IP:5001
```

### Nenhum Agente Aparecendo

**Problema:** O painel carrega mas a barra lateral de agentes está vazia

**Soluções:**

1. **Criar seu primeiro agente:**
   ```bash
   curl -X POST http://localhost:8000/api/agents \
     -H "Content-Type: application/json" \
     -d '{"id": "dev", "name": "Agente Dev", "role": "developer", "avatar": "💻", "status": "idle"}'
   ```

2. **Importar da configuração do OpenClaw:**
   - Clique em "Importar do OpenClaw" no Gerenciamento de Agentes
   - Requer `~/.openclaw/openclaw.json` com agentes configurados

3. **Verificar integração com OpenClaw:**
   ```bash
   # Verificar se a configuração existe
   ls ~/.openclaw/openclaw.json

   # Verificar endpoint da API
   curl http://localhost:8000/api/openclaw/status
   ```

### Problemas com Banco de Dados

**Problema:** Tarefas/agentes não persistindo ou erros de banco de dados

**Soluções:**

1. **Verificar arquivo do banco de dados:**
   ```bash
   # Localização padrão
   ls backend/data/mission_control.db

   # Criar diretório se ausente
   mkdir -p backend/data
   ```

2. **Resetar banco de dados:**
   ```bash
   rm backend/data/mission_control.db
   # Reiniciar backend - o banco de dados será recriado automaticamente
   ```

3. **Permissões:**
   ```bash
   chmod 755 backend/data
   chmod 644 backend/data/mission_control.db
   ```

### Falha na Conexão WebSocket

**Problema:** O painel mostra "Falha na Conexão" ou sem atualizações em tempo real

**Soluções:**

1. **Verificar se o backend está rodando:**
   ```bash
   curl http://localhost:8000/api/stats
   ```

2. **Verificar endpoint WebSocket:**
   ```bash
   # Deve mostrar resposta de upgrade
   curl -i -N -H "Connection: Upgrade" -H "Upgrade: websocket" \
        http://localhost:8000/ws
   ```

3. **Erros no console do navegador:**
   - Abra DevTools → Console
   - Procure erros de conexão WebSocket
   - Causa comum: backend não rodando ou porta errada

### Status do Agente Não Atualizando

**Problema:** Agentes presos em "OFFLINE" ou status não muda

**Soluções:**

1. **Verificar arquivos de sessão do OpenClaw:**
   ```bash
   # Verificar se o diretório de sessão existe
   ls ~/.openclaw/agents/AGENT_ID/sessions/

   # Verificar atividade recente
   find ~/.openclaw/agents/*/sessions -name "*.jsonl" -newermt "1 hour ago"
   ```

2. **Atualização manual de status:**
   ```bash
   curl -X PATCH "http://localhost:8000/api/agents/AGENT_ID/status?status=WORKING"
   ```

3. **Atualizar lista de agentes:**
   - Clique no botão de atualizar na barra lateral de agentes

---

## Contribuindo

Contribuições são bem-vindas! Por favor, abra issues e pull requests no repositório deste fork:
[leandromarcosmoreira/ClawController](https://github.com/leandromarcosmoreira/ClawController)

Para contribuir com o projeto original, acesse: [mdonan90/ClawController](https://github.com/mdonan90/ClawController)
