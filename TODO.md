# TODO

- `Execution Trace` must have "--" after `Data (Base58): "2"` to separate line
- refer to
  ```
  reev_lib::session_logger::ToolCallInfo` (used in session logs) - has `tool_id` field
  reev_lib::agent::ToolCallInfo` (used in flow tracking) - has `tool_name` field
  ```
  clean this mess, use tool_id instead of tool_name, grep
- clean TODO

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
