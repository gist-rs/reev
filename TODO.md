- Dokerfile with preload surfpool specfific verison by `.env`, we already have this surfpool loder in the code and it's gonna be better if we prelaod via Doker and use code to check for extracted binary and use current code as a fallback in case we not run via Docker. Anyhow this code should respect same specfific verison by `.env` and throw error yell for either docker, or manually run surfpool service via `https://docs.surfpool.run/install` if fallback load github didn't work.

-
you should keep an eye on this logs/reev-agent.log so you can fix tool call, it work before not sure reflection help: [@REFLECT.md](file:///Users/katopz/git/gist/reev/REFLECT.md) , but i think tool desc and prompt also context we add should fix this, it should get any wallet info because we must provide it so maybe you must ensure that info is included in req


- you can run benchmarks/114-jup-positions-and-earnings.yml to test only position
- you maybe need to create 117 to call that tokens tool under crates/reev-agent/examples


- bring your own agents: will call with only prompt (you prepare the account, no token = fail)
- bring your own model api_key (you prepare api_url, api_key, we will send you the request as see in benchmarks suite)
- bring your own prompt: we will run our agent against your prompt (currently only jupiter support, if you add prompt that we dont support we will take a look and see which protocol mentino the most)
