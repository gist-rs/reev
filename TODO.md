# TODO (skip this doc, this meant for human tasks, dont read or write)

commit and DRY [@openai.rs](zed:///agent/file?path=%2FUsers%2Fkatopz%2Fgit%2Fgist%2Freev%2Fcrates%2Freev-agent%2Fsrc%2Fenhanced%2Fopenai.rs) and [@zai_agent.rs](zed:///agent/file?path=%2FUsers%2Fkatopz%2Fgit%2Fgist%2Freev%2Fcrates%2Freev-agent%2Fsrc%2Fenhanced%2Fzai_agent.rs), create common helper

---
refer to

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

can you check that we use it correct everywhere, grep for that

---

plz note do and dont in the head of this file, you always go the wrong way for 10 times, we lost 3 days already for this feature.

rig framework allow us to inject provider and intercept response like we did so we should able to mod and back to the flow without mod the rig lib directly.

---


currently we mak a custom tool calling for glm and that cumbersome and not same otel.

refer to
i think we can impl new glm provider like this:
and modify the response to make it openai compatible.
then using it as custom provider like this

what you think?


- llm is not allow to generate tx


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
