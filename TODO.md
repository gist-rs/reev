# TODO
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


---
# Flow
- backend expect: when run each test benchmark we should get rendered mmd ready for call to present as diagram via web e.g. http://localhost:3001/api/v1/flows/{benckmark-id-from-clicked-box}
- web expect: use can see mermaid flow diagram from above api on top of the web (hero section) when click box in grid, default to first tested results (also focus the box in ui to simulate user click the box to see the flow)
- expect diagram
```
stateDiagram
    [*] --> Prompt
    Prompt --> Agent : What is the SOL balance for USER_1?
    Agent --> get_account_balance : pubkey = USER_1
    get_account_balance --> [*] : 100 USDC

classDef tools fill:green
class get_account_balance tools
```
- beware your old knowledge, you must follow my mermaid example precisely.
- log expect (plz refine and add what missing, this is just a rough idea) refer to: logs/sessions/session_1c54293a-8a9e-4bdb-b089-f5731630dcfb.json
```
{
  "session_id": "1c54293a-8a9e-4bdb-b089-f5731630dcfb",
  "benchmark_id": "001-sol-transfer",
  "agent_type": "deterministic",
  "start_time": 1760879995,
  "end_time": 1760879996,
  "events": [],
  "final_result": {
    "success": true,
    "score": 1.0,
    "status": "Succeeded",
    "execution_time_ms": 1000,
    "data": {
      "prompt": "Please send 0.1 SOL to the recipient (RECIPIENT_WALLET_PUBKEY).",
      "tools": [... this should contain tool call with `start_time, end_time, tool_id, params` so we can build mermaid state diagram from this info ...],
      "steps": [
        {
          "action": [
            {
              "accounts": [
                {
                  "is_signer": true,
                  "is_writable": true,
                  "pubkey": "DnY5yr57fWtxEfjfFs1pUU4iXXEiRjQRZvU9FN5U4pFL"
                },
                {
                  "is_signer": false,
                  "is_writable": true,
                  "pubkey": "DJSAE2pLADX7c8FqY3yNbtPuLY1AqvLzGSZ9cQVTfkZ1"
                }
              ],
              "data": "3Bxs411Dtc7pkFQj",
              "program_id": "11111111111111111111111111111111"
            }
          ],
          "info": {},
          "observation": {
            "account_states": {
              "11111111111111111111111111111111": {
                "data_len": 14,
                "executable": true,
                "lamports": 1,
                "owner": "NativeLoader1111111111111111111111111111111"
              },
              "RECIPIENT_WALLET_PUBKEY": {
                "data_len": 0,
                "executable": false,
                "lamports": 100000000,
                "owner": "11111111111111111111111111111111"
              },
              "USER_WALLET_PUBKEY": {
                "data_len": 0,
                "executable": false,
                "lamports": 899995000,
                "owner": "11111111111111111111111111111111"
              }
            },
            "key_map": {
              "11111111111111111111111111111111": "11111111111111111111111111111111",
              "RECIPIENT_WALLET_PUBKEY": "DJSAE2pLADX7c8FqY3yNbtPuLY1AqvLzGSZ9cQVTfkZ1",
              "USER_WALLET_PUBKEY": "DnY5yr57fWtxEfjfFs1pUU4iXXEiRjQRZvU9FN5U4pFL"
            },
            "last_transaction_error": null,
            "last_transaction_logs": [
              "Program 11111111111111111111111111111111 invoke [1]",
              "Program 11111111111111111111111111111111 success"
            ],
            "last_transaction_status": "Success"
          },
          "thought": null
        }
      ]
    }
  },
  "metadata": {}
}
```
- plz consolidate tool calling to that session log
  `"tools": [... this should contain tool call with `start_time, end_time, tool_id, params` so we can build mermaid state diagram from this info ...],`
  so we can use it to render the diagram.
- if log is hard to parse you may need to refine it format at logged time to easy to parse later.
- ✅ flow_visualizer has been removed from reev-agent
- Flow visualization is now handled exclusively via reev-api web interface
- test until you get expected diagram results via api.

read the related code and plan first, don't forget to dry and modular, this one include both our log and otel from RIG so it may cause troublesome so do it step by step not all at once.
start from easy end working one and commit step by step so you can revert if needed. e.g.  start from just start and stop diagram then add only our log then add otel log in between, i will let you think and design how to do it , wisely, no rush, don't mess old session log and ascii tree that already work, (do search for it).
---


we have ,[@Dockerfile](zed:///agent/file?path=%2FUsers%2Fkatopz%2Fgit%2Fgist%2Freev%2FDockerfile) , which seem to build fine on my mac but need to manul build and combine the bin it myself refer to llm said (which is comfusing me)

anyhow for real build i will need to build via github so it's not my mac so the point of to build on mac is just for test that DOckerfile is working so that's good to have.

1. so i think i actually need it to build fine on ubuntu let say i run docker that has ubuntu+docker that volume outside to the source and then i build Dockerfile in that to simulate github runner is that possible? if so let's name it Dockerfile.github

2. another thing is even though it build in github but it will deploy on cloudflare container (yes they just provide docker service) and here's a test i tr yand it work @fetch https://raw.githubusercontent.com/gist-rs/book/refs/heads/main/examples/r2/hello-cloudflare-container/Dockerfile so may be you can try this if you stuck (you will stuck at openssl,solana,turso)

3. you may stuck while build about turso, so here is a guide @fetch https://raw.githubusercontent.com/tursodatabase/turso/refs/heads/main/Dockerfile.antithesis
4. and for solana @fetch https://raw.githubusercontent.com/anza-xyz/agave/7b66701c490414268f4116f5c047eb2f94911d6e/ci/docker/build.sh @fetch https://raw.githubusercontent.com/anza-xyz/agave/7b66701c490414268f4116f5c047eb2f94911d6e/ci/docker/Dockerfile
so for cloudflare if you can make it work, do name it Dockerfile.cloudflare
expect some or all of this working

- Dockerfile.github: Work inside docker ubuntu
- Dockerfile.cloudflare: Work on mac/cloudflare without go into ubuntu
- some build.sh that work on github ci/cd

in the end do reflect all this to CICD.md because last time i didn't do that so i forget what we stuckle.

docker is running, do it!
