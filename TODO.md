# TODO
- Dokerfile with preload surfpool specfific verison by `.env`, we already have this surfpool loder in the code and it's gonna be better if we prelaod via Doker and use code to check for extracted binary and use current code as a fallback in case we not run via Docker. Anyhow this code should respect same specfific verison by `.env` and throw error yell for either docker, or manually run surfpool service via `https://docs.surfpool.run/install` if fallback load github didn't work.

- we must find the way to monitor the tool calling so we can score agent for `tool calling` too (must call tool precisly, you can imagine what tools should call in order and scoring from that). i can see log at logs/reev-agent.log but not sure how we can collect that to scoring, any idea?

# TOFIX
- why we need `My wallet is USER_WALLET_PUBKEY.`? the context should included by say `connected wallet pubkey: {generated_pubkey}`
-

## Complex Flows
- more complex flow in same protocols e.g. swap if not enough then withdraw and deposit. // we should have all combination as possible.
- interact between 2 or more protocols e.g. withdraw from kamino and led via jupiter.

## Dashboard

1. bring your own agents: will call with only prompt (you prepare the account, no token = fail)

2. bring your own model api_key (you prepare api_url, api_key, we will send you the request as see in benchmarks suite)

3. bring your own prompt:
   3.1 jupiter scope: we will convert your prompt to yml and run it on our system.
   3.2 outof scope: we will run our agent against your prompt (currently only jupiter support, if you add prompt that we dont support we will take a look and see which protocol mentino the most)

4. bring nothing: we will show Dashboard jup x [coding, local-qwen3, gemini, grok4, glm4.6]


## UI

- ready to change the number randomly from tempate e.g. "Swap ___ SOL for USDC"
