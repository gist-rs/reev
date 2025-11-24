# TODO (skip this doc, this meant for human tasks, dont read or write)

---

this is not scale `// Check if the prompt contains transfer keywords to set appropriate default`
this is look wrong? `// Extract recipient from the prompt`
`let recipient_pubkey = if let Some(address_start) = prompt.find("gistme") {` // this is really bad coding
why there is a huge log `INFO execute_flow` for simple transfer sol?

---

[@PLAN_CORE_V2.md](file:///Users/katopz/git/gist/reev/PLAN_CORE_V2.md) , [@ISSUES.md](file:///Users/katopz/git/gist/reev/ISSUES.md), let's fix [@end_to_end_swap.rs](file:///Users/katopz/git/gist/reev/crates/reev-core/tests/end_to_end_swap.rs) to complete 
---
1. prompt "swap 1 SOL for USDC"
2. i should see log info for yml prompt (with wallet info get from surfpool) that send to glm-coding (ZAI_API_KEY and already set at .env, do use it with dotenvy)
3. i should see log info for swap tool calling from llm
4. i should see tx that gen from that tool.
5. it should sign that tx with default keypair `~/.config/solana/id.json`.
6. i should see tx complete res from surpool
7. create other test prompt "sell all SOL for USDC" and it should repeat 1. step with user wallet context as (this mean the test code should share common and DRY and ready for any input later)
---
by using [@SURFPOOL.md](file:///Users/katopz/git/gist/reev/SURFPOOL.md) , not mock. ask me for confirm before impl if complex and break the plan

---

commit current state against the plan [@PLAN_CORE_V2.md](file:///Users/katopz/git/gist/reev/PLAN_CORE_V2.md) and refine [@ISSUES.md](file:///Users/katopz/git/gist/reev/ISSUES.md), and handover detail in [@TASKS.md](file:///Users/katopz/git/gist/reev/TASKS.md) 

---

Show me integration test for

1. prompt "swap 1 SOL for USDC"
2. i should see log info for yml prompt (with wallet info get from surfpool) that send to glm-coding (ZAI_API_KEY and already set at .env, do use it with dotenvy)
3. i should see log info for swap tool calling from llm
4. i should see tx that gen from that tool.
5. it should sign that tx with default keypair `~/.config/solana/id.json`.
6. i should see tx complete res from surpool
7. create other test prompt "sell all SOL for USDC" and it should repeat 1. step with user wallet context as (this mean the test code should share common and DRY and ready for any input later)

if this kind of integration test not exist yet plz add and do make a real call to api, you can add ignore for this test later

not sure it should land in crates/reev-orchestrator/tests
or crates/reev-core/tests

if you dont know surfpool, do grep from existing crates/reev-core
---

can you help cross check and refine/fix if needed?

[@swap_flow_integration_test.rs](file:///Users/katopz/git/gist/reev/crates/reev-orchestrator/tests/swap_flow_integration_test.rs) 

it must use default keypair which is ~/.config/solana/id.json btw


---

refer to [@PLAN_CORE_V2.md](file:///Users/katopz/git/gist/reev/PLAN_CORE_V2.md) ,[@SOLANA_KEYPAIR.md](file:///Users/katopz/git/gist/reev/SOLANA_KEYPAIR.md), [@ISSUES.md](file:///Users/katopz/git/gist/reev/ISSUES.md), [@TASKS.md](file:///Users/katopz/git/gist/reev/TASKS.md) , did we fin the task and issue, be honest.

---

Show me integration test for
1. prompt "swap 1 SOL for USDC"
2. i should see log info for yml prompt (with wallet info get from surfpool) that send to glm-coding
3. i should see log info for swap tool calling from llm
4. i should see tx that gen from that tool.
5. it should sign thst tx with default keypair `~/.config/solana/id.json`.
6. i should see tx complete res from surpool

if this integration test not exist yet plz add and do make a real call to api, you can add ignore for this test later

not sure it should land in crates/reev-orchestrator/tests
or crates/reev-core/tests

if you dont know surfpool, do grep from existing crates/reev-core, no mock allow, this is integration test

---

fix issue, re-test, re-check code, impl remain tasks, rm done task, ALWAYS RUN SERVER IN BG

refer to the plan, impl it step by step, feel free to remove old code/test in existing crate if it throw errors while impl because it obsolete and we dont use it anymore, and/or consider delete unused in all crate if new reev-core doesn't need it to compact the code. no migration needed no mercy no need to keep compatible.

---
we already proof all that and suddenly failed after add pingpong tho. i want to focus on

1. user_prompt "use my 50% sol to multiply usdc 1.5x on jup"
2. system detect USER_WALLET_PUBKEY and replace with generated pubkey filled with some SOL.
3. system provide user's wallet info including token info with price into token_context and record wallet state to db use wallet address as index and request_id for ref as enter state.
4. system get all tools with description aka tool_context.
5. system prepare refined_instruct that said "refine user prompt to match tools description:{tool_context}"
6. system send that token_context+refined_instruct+user_prompt to glm-4.6-coding
7. llm agent refine that to series of prompt that match tool desc e.g. [ "swap 1 SOL to USDC", "lend {SWAPPED_USDC} to jupiter" ]
8. manager pick that token_context, prompt_series and call llm with tool calling info one by one
9. manager process result from llm which must be tool calling only, record tool_name, tool_params to reev.db each row index by request_id uuidv7 (uuidv7 has time sortable id)
11. manager call tool as llm filled params with token_context.
12. manager get tool results as tx from jupiter api.
13. manager record tx to reev.db each row index by request_id.
14. manager use executor to call surfpool to process that tx.
15. manager collect results (both failure and succeed) from executor to reev.db in new table with same request_id for ref.
16. manager build new context for next tool e.g. current_context="
original_wallet_info:
  - SOL
    - addresss:
    - amount: 1
    - price_usd: 161
  - USDC
    - pubkey; EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v
    - amount: 10
    - price_usd: 1
# after swap (we can add prev tool call name here)
current_wallet_info:
  - SOL
    - addresss:
    - amount: 0
  - USDC
    - pubkey; EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v
    - amount: 171 # swap +161 USDC from 1 SOL
"
(refine this to yml format and add missing info)
// for prompt "lend {USDC_AMOUNT} to jupiter" and let llm reasoning about that fill that itself, expect USDC_AMOUNT = 161 because original we have 10 so comment is the key here to let llm understand
17. Now we repeat #9 until all steps process or failed
18. manager call wallet info and record wallet state to db use wallet address as index and request_id for ref as exit state.

can you help refine
- we may create reev-core and start over from that
- feel free to copy old fn or use exiting lib but we have to copy code to new core so we don't make compile error for old one and get confuse by that.
- all prompt must be in yml so we can parse later.
- no json allow in db, must be structed and typed only.
- log is confusing and overwhelm ai-coding(you) lately , keep all state (inclduing prompt) write to db for each step as plan and do add more what i missing, in the end we shouldable to see all each state in db ready for debug, create flow and scoring later
- no migration need, we start from the ground up
---

i exepect benchmarks/300-jup-swap-then-lend-deposit-dyn.yml work with api dynamic flow ping png via glm-4.6-coding

```
curl -s -X POST http://localhost:3001/api/v1/benchmarks/execute-direct \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "use my 50% sol to multiply usdc 1.5x on jup",
    "wallet": "USER_WALLET_PUBKEY",
    "agent": "glm-4.6-coding",
    "shared_surfpool": false,
    "benchmark_id": "300-jup-swap-then-lend-deposit-dyn"
  }' | jq '.result.tool_calls'
```

 , check that the code align with the plan or not, check result step by step, jsonl then yml then db then read db to mermaid diagram, dont skip, think harder, expect benchmarks/300-jup-swap-then-lend-deposit-dyn.yml working with yml or just prompt via glm-4.6-coding and see mermaid statediagram as a plan

[@AGENTS.md](file:///Users/katopz/git/gist/reev/AGENTS.md) ,[@DYNAMIC_BENCHMARK_DESIGN.md](file:///Users/katopz/git/gist/reev/DYNAMIC_BENCHMARK_DESIGN.md) ,[@HANDOVER.md](file:///Users/katopz/git/gist/reev/HANDOVER.md) ,
ai is overclaim that llm is complete but messup rulesbase test only, ignore the rulesbase test because we want llm flow to work first

and rule-based is only for deterministic agent btw, ?mock=true grep for that and by the plan you will need feature flag for that to see DYNAMIC_BENCHMARK_DESIGN.md

let's test llm base for 300-jup-swap-then-lend-deposit-dyn.yml via api [@DEV_FLOW.md](file:///Users/katopz/git/gist/reev/DEV_FLOW.md) , it should work with dynamic llm gen static flow/step and get mermaid diagram+score

---


check that reev-orchestrator did allowed tool by llm or not, we must use llm at gateway not rules based

1. change to all tool is allowed because current logic is to strict and have many bug around that.
2. static yml like 001-sol-transfer.yml may add allowed tools.
3. always add exist tool so it's not lock up.
4. for 300-jup-swap-then-lend-deposit-dyn.yml it should generate allowed tools by llm like #2 to go in the same flow.
5. remember that dynamic is just llm gen static.
6. use rig/rig-core/examples/calculator_chatbot.rs as example for add tool.

---

format!("{}/lend/v1/earn/positions", self.api_base_url)
DRY

--

remove completed issuses md and add this enum refactor to new issue md

---

expect
Prompt → jupiter_swap → jupiter_lend → [*]

actual
Prompt → jupiter_lend_earn_deposit → [*]

the condition is use has some sol and need swap to usdc first then deposit
it should work like benchmarks/200-jup-swap-then-lend-deposit.yml

200 = static yml -> flow
300 = dynamic - llm -> static yml -> flow

---

refer to 300-jup-swap-then-lend-deposit-dyn.yml
tags: ["dynamic", "multiplication", "jupiter", "yield", "strategy"]
flow_type: "dynamic")

flow_type should detemine from tags is good enough // so if tags "dynamic" exist -> flow_type=dynamic

also

```
        "300-jup-swap-then-lend-deposit-dyn" => {
            info!(
                "[reev-agent] Matched '300-jup-swap-then-lend-deposit-dyn' id. Starting dynamic flow."
            );

            let user_pubkey_str = key_map
                .get("USER_WALLET_PUBKEY")
                .context("USER_WALLET_PUBKEY not found in key_map")?;
            let user_pubkey = Pubkey::from_str(user_pubkey_str)?;
            let _user_pubkey = Pubkey::from_str(user_pubkey_str)?;

            // Dynamic flow - agent will use Jupiter tools to execute the multiplication strategy
            info!("[reev-agent] Dynamic flow: Agent will execute 50% SOL to USDC swap then lend for 1.5x multiplication");

            // For dynamic benchmarks, return empty instructions and let LLM agent handle the execution
            let flow_response = serde_json::json!({
                "benchmark_id": "300-jup-swap-then-lend-deposit-dyn",
                "agent_type": "dynamic",
                "mode": "llm_execution",
                "strategy": "use jupiter tools to multiply USDC position by 1.5x using 50% of SOL"
            });
            Ok(serde_json::to_string(&flow_response)?)
        }
```
is redandant about "dynamic" and should determine from flow_type=dynamic

---

expect working mermaid flow completed info with 300-jup-swap-then-lend-deposit-dyn.yml, via api glm-4.6-coding

up tasks, issue md then commit and stop

---

let's add that to DYNAMIC_BENCHMARK_DESIGN.md
sperate tasks to make it happen in TASKS.md
focus on make it work step by step so less confuse and decouple between each task as possible.
the main idea is

1. keep the code as md design (scan code and before tasks)
2. separate bench and user but same core logic, focus on bench first
3. no more mock data or cheating, if it failed just let it failed and get bad score at that step
4. the tool name currently use string and fuck up a lot, try enum string aka strum crate

---

1. refine PLAN_DYNAMIC_FLOW.md
2. refine DYNAMIC_BENCHMARK_DESIGN.md
3. refine/compact tasks.md


[@AGENTS.md](file:///Users/katopz/git/gist/reev/AGENTS.md), [@DEV_FLOW.md](file:///Users/katopz/git/gist/reev/DEV_FLOW.md) ,[@DYNAMIC_BENCHMARK_DESIGN.md](file:///Users/katopz/git/gist/reev/DYNAMIC_BENCHMARK_DESIGN.md) ,[@HANDOVER.md](file:///Users/katopz/git/gist/reev/HANDOVER.md), [@TASKS.md](file:///Users/katopz/git/gist/reev/TASKS.md), [@ISSUES.md](file:///Users/katopz/git/gist/reev/ISSUES.md) ,[@PLAN_DYNAMIC_FLOW.md](file:///Users/katopz/git/gist/reev/PLAN_DYNAMIC_FLOW.md) , impl issues, tasks, ALWAYS RUN SERVER IN BG, focus on `glm-4.6-coding` only

---

the problem is rig framework use otel, that's why de design like that, somehow you avoid that design decision, create issue, this is critical misconcept

the problem is openai flow use rig framework and otel, that's why de design like that, it work now because we mod zai agent req/res so we can capture but for openai and other agent in the future, we will have to impl all this all over again and there's is no reason to use rig at that point if we keep doing that.

but we control orchestrator so is fine for orchestrator part but
1. Agent → Orchestrator → JSON → YML → DB → YML Parser → Mermaid // because yml has better comment and newline
2. Did `Agent → Orchestrator → JSON` will work for openai agent flow? // if yes it's fine
3. All other tool call is in rig+otel, what your plan for this? if you tend to use direct way? consolidate each flow/step as chuck later?


```
Agent Execution → Ping-Pong Orchestrator → Direct JSON → Database → JSON Parser → Mermaid
                               ↓
                         (No OTEL involved)
```

```
Agent → Orchestrator → JSON → DB → JSON Parser → Mermaid
```

```
Agent → OTEL Traces → enhanced_otel.jsonl → JsonlToYmlConverter → OTEL YML → SessionParser → Mermaid
```
---

---
i exepect benchmarks/300-jup-swap-then-lend-deposit-dyn.yml work with dynamic flow ping png via glm-4.6-coding
---

RUST_LOG=info cargo run -p reev-runner -- benchmarks/001-sol-transfer.yml --agent glm-4.6-coding
---
Caused by:
    LLM API request failed with status 500 Internal Server Error: {"error":"Internal agent error: ZAI model 'glm-4.6-coding' validation failed: invalid authentication. Please check if the model is available and your API credentials are correct."}


[@ARCHITECTURE.md](file:///Users/katopz/git/gist/reev/ARCHITECTURE.md)

---
GLM_CODING_API_URL="https://api.z.ai/api/coding/paas/v4"
ZAI_API_URL="https://api.z.ai/api/paas/v4"

---
has been set in .env

glm-4.6 must use ZAI_API_URL="https://api.z.ai/api/paas/v4"
curl --location 'https://api.z.ai/api/paas/v4/chat/completions' \
--header 'Authorization: Bearer YOUR_API_KEY' \
--header 'Accept-Language: en-US,en' \
--header 'Content-Type: application/json' \
--data '{
    "model": "glm-4.6",
    "messages": [
        {
            "role": "user",
            "content": "Hello"
        }
    ]
}'
glm-4.6-coding must use GLM_CODING_API_URL="https://api.z.ai/api/coding/paas/v4"
curl --location 'https://api.z.ai/api/coding/paas/v4/chat/completions' \
--header 'Authorization: Bearer YOUR_API_KEY' \
--header 'Accept-Language: en-US,en' \
--header 'Content-Type: application/json' \
--data '{
    "model": "glm-4.6",
    "messages": [
        {
            "role": "user",
            "content": "Hello"
        }
    ]
}'

glm-4.6 and glm-4.6-coding use same `"model": "glm-4.6 ",`

glm-4.6 must use openai compatible crates/reev-agent/src/enhanced/openai.rs
glm-4.6-coding must use crates/reev-agent/src/enhanced/zai_agent.rs

---

cargo run -p reev-runner -- benchmarks/001-sol-transfer.yml --agent glm-4.6
cargo run -p reev-runner -- benchmarks/001-sol-transfer.yml --agent glm-4.6-coding

---

Test this and see flow work or not via api curl only (not cli) focus on agent glm-4.6 only
1. 001-sol-transfer.yml
2. 300-jup-swap-then-lend-deposit-dyn.yml

expect flow working e.g. http://localhost:3001/api/v1/flows/306114a3-3d36-43bb-ac40-335fef6307ac

---

ALWAYS RUN SERVER IN BG

create handover.md

refer to above can you check the code, update md if need and continue impl?

---

do we doing this rn for runner part?

- before: `manual -> static yml -> runner`
- after: `reev-orchestrator -> dynamic yml -> runner`

see yml before send to runner will be easier to see and debug and runner behave to same?

---

- DRY `reev_db::types::AgentPerformance {...}`
- DRY `Successfully synced`

---

- did this schema ready for scoring how good llm call the tool with times taken, low tools call with low latency should have good score (refine/suggest me on this metrics) do search code/md for score/scoring and learn from current scoring first,

---

refer to

```
Here are the common settings for tool_choice:
"auto" (Default): The model autonomously determines whether to call a tool and, if so, which tool to call based on the user's prompt and the available tool descriptions. This is the standard behavior for most conversational interactions.
"none": This explicitly prevents the model from calling any tools. The model will only generate a text-based response.
"required": This setting forces the model to call a tool. The model will then select which tool to call from the available options. This can be useful in scenarios where tool usage is essential for the application's logic, but it can also lead to infinite loops if not handled carefully, especially in streaming or real-time contexts where the model might continuously try to invoke tools without generating a user-facing response.
Specific Tool Object: You can also specify a particular tool to be called by providing a tool object, such as:
Code

    {
      "type": "function",
      "function": {
        "name": "my_specific_tool"
      }
    }
This forces the model to call the designated tool, regardless of its own assessment of the prompt.
Considerations:
Using "required" or forcing a specific tool can be powerful but requires careful management of the conversation flow to avoid repetitive or non-responsive behavior from the model.
In streaming or real-time applications, setting tool_choice to "required" on every turn without allowing for model responses can create a loop of tool invocations.
The tool_choice parameter is crucial for building robust applications that integrate large language models with external functionalities.
```

can you check that we use it correctly everywhere? grep for that

---

- Dokerfile with preload surfpool specfific verison by `.env`, we already have this surfpool loder in the code and it's gonna be better if we prelaod via Docker and use code to check for extracted binary and use current code as a fallback in case we not run via Docker. Anyhow this code should respect same specfific verison by `.env` and throw error yell for either docker, or manually run surfpool service via `https://docs.surfpool.run/install` if fallback load github didn't work.

-`"Please send 15 USDC from my token account (USER_USDC_ATA) to the recipient's token account (RECIPIENT_USDC_ATA)."` look not like human conversation, it should say `"Send 15 USDC to xxx." which xxx is someone wallet and we should provide the wallet info including ata by code inject to context for llm.
  - user prompt `"Send 15 USDC to xxx."
  - llm get balance info for that token via tool (did we have this tool yet?) and inject to the context so it's user_prompt+wallet_info
  - llm call remain tools maybe swap and reason about current state e.g. retry once or give up if condition not sttified e.g. no balance or high slippage
  - we collect all the flow to score that (because we aim to evalate the flow and tx) // we have this already just need cross check
  - create yml report for debug and report what llm do and how tx doing, ready for make a report and bechmark // we have this already just need cross check

## Complex Flows
- more complex flow in same protocols e.g. swap if not enough then withdraw and deposit. // we should have all combination as possible.
- interact between 2 or more protocols e.g. withdraw from kamino and led via jupiter.

## Dashboard

- add agent tool call as mermaid state diagram https://mermaid.js.org/syntax/stateDiagram.html

1. bring your own agents: will call with only prompt (you prepare the account, no token = fail)

2. bring your own model api_key (you prepare api_url, api_key, we will send you the request as see in benchmarks suite)

3. bring your own prompt:
   3.1 jupiter scope: we will convert your prompt to yml and run it on our system.
   3.2 outof scope: we will run our agent against your prompt (currently only jupiter support, if you add prompt that we dont support we will take a look and see which protocol mentino the most)

4. bring nothing: we will show Dashboard jup x [coding, local-qwen3, gemini, grok4, glm4.6]

## UI

- ready to change the number randomly from tempate e.g.
  - user: "Swap ___ [SOL] for [USDC]" → agent: "Done! You now have 222 USDC" // This met required params
  - user: "Sell ___ [SOL]" → agent: "To ___?" // This missing target token so agent ask to fillfilments

## Refactor
- use const for any address e.g. `ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL`
