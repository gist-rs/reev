Re-Eval 🪸

# 

# **A Framework for the Reproducible Evaluation of Solana-Native LLM Agents**

## **Part I: Foundational Principles for Agent Evaluation**

The evaluation of Large Language Models (LLMs) has matured significantly, with established benchmarks and metrics assessing core competencies like language understanding, knowledge factuality, and reasoning.1 However, these methodologies are fundamentally insufficient for a new class of systems: LLM-based autonomous agents. An agent is not merely a text generator; it is a system that perceives a dynamic environment, formulates plans, and executes actions to achieve a goal.3 When the environment is a high-stakes, stateful system like the Solana blockchain, the evaluation paradigm must shift from assessing static outputs to analyzing dynamic behavior.

This report specifies a comprehensive and reproducible framework for evaluating custom LLM agents designed to operate on the Solana network. The architecture is grounded in established standards for reinforcement learning and agent-environment interaction, tailored specifically to the unique challenges and primitives of the Solana ecosystem. The ultimate goal is to provide a methodology that is not only technically rigorous but also verifiable and trustworthy, meeting the high standards required for deployment in decentralized applications and for review by entities such as the Solana Foundation. The explicit reference to a solana-gym-env project, though inaccessible, strongly indicates an institutional interest in aligning with robust, community-accepted standards for agent evaluation, a signal that informs the core design principles of this framework.4

### **1.1 Beyond Text Generation: The Agentic Paradigm**

Evaluating a standalone LLM is analogous to testing a car's engine on a dynamometer; it measures power, efficiency, and emissions in a controlled, isolated setting. In contrast, evaluating an LLM-based agent is akin to road-testing the entire vehicle under a variety of real-world conditions—navigating city streets, highways, and adverse weather.3 The focus shifts from the engine's isolated performance to the car's overall effectiveness in achieving its purpose: transportation. Similarly, an agent's value is not determined by the quality of its internal monologue or "thought process" alone, but by its ability to successfully complete tasks in its operational environment.5

Agentic systems are characterized by a continuous loop of perception, planning, and action. They often leverage external tools to augment their capabilities, such as executing code, searching the web, or, in this context, interacting with a blockchain.7 This introduces several new dimensions that must be evaluated beyond traditional NLP metrics 3:

* **Task Completion Rate:** The most fundamental metric. Did the agent successfully achieve the overall goal specified by the user?  
* **Reasoning and Planning Quality:** Was the agent's strategy logical? Did its intermediate decisions and tool selections make sense in the context of the task?  
* **Tool Selection and Invocation:** Did the agent choose the appropriate tools (e.g., Solana programs) for sub-tasks, and were the parameters supplied to those tools correct?  
* **Efficiency:** How many steps, API calls, or how much time and resources (e.g., transaction fees) did the agent require to complete the task?  
* **Adaptability and Robustness:** How well does the agent handle errors, unexpected feedback from the environment (e.g., failed transactions), or changes in task requirements?

These dimensions highlight that the object of evaluation is the entire system's behavior over time, not an isolated input-output pair. The agent's ability to navigate complexity, manage state, and recover from failure are paramount, especially in a blockchain environment where actions are often irreversible and have real economic consequences.9 Therefore, the evaluation framework must be designed to capture and measure this sequential, interactive behavior in a structured and repeatable manner.

### **1.2 Why Gymnasium is the Gold Standard for Reproducible Interaction**

To systematically evaluate agentic behavior, a standardized interface between the agent and its environment is required. The Gymnasium library (a maintained fork of OpenAI's Gym) provides this gold standard.11 It is a language-agnostic API designed to represent general reinforcement learning and agent-based problems, making it the ideal foundation for the

Solana-Gym evaluation environment.

The core of the Gymnasium framework is the Env class, which encapsulates the environment's dynamics. The interaction is governed by a few key methods and attributes 13:

* reset(): Resets the environment to a well-defined initial state and returns the first observation. Crucially, this method accepts a seed argument, which is the cornerstone of reproducibility.  
* step(action): The agent submits an action to the environment. The environment processes the action, updates its internal state, and returns a tuple containing the next observation, a reward signal, a terminated flag (indicating task completion), a truncated flag (indicating an external cutoff), and an info dictionary for diagnostic data.  
* render(): Provides a visualization of the environment's current state, with modes like human, rgb\_array, or ansi for text-based output.  
* action\_space: Defines the set of all valid actions the agent can take.  
* observation\_space: Defines the structure and bounds of the data the agent receives from the environment.

Adopting the Gymnasium API provides several profound advantages. First, it imposes a clear separation of concerns between the agent's logic and the environment's dynamics. The agent developer can focus on the decision-making model, while the environment developer focuses on accurately simulating the Solana blockchain. Second, it provides a structure that is widely understood in the AI research community, facilitating collaboration and comparison with other work.

Most importantly, the Gymnasium standard directly addresses the central challenge of evaluating a probabilistic system (the LLM) interacting with a deterministic one (the blockchain) in a reproducible way. LLM outputs are inherently non-deterministic, while a blockchain is a deterministic state machine. This creates a paradox: how can an evaluation be reproduced if the agent's behavior might change on every run? The reset(seed=...) method is the solution. By setting a specific seed, two things are achieved:

1. The environment's internal pseudo-random number generator is initialized to a deterministic state. This ensures that any stochastic elements within the simulation (if any) are repeatable.  
2. The agent's own pseudo-random number generator can also be seeded with the same value. This forces the LLM to produce a deterministic sequence of actions for that specific seed.

This means that an evaluation run for a given seed is perfectly reproducible. A third party, such as the Solana Foundation, can take the exact same agent code, benchmark, and seed, and get the exact same result, down to the last transaction. By running the evaluation across a suite of different seeds, one can sample the agent's performance distribution, providing a statistical overview of its capabilities while ensuring that each individual data point is fully verifiable. This resolves the reproducibility paradox and forms the non-negotiable foundation of this framework's design.

### **1.3 A Taxonomy of Evaluation: Objectives and Processes**

To ensure the evaluation is comprehensive and well-structured, it is useful to adopt a formal taxonomy that distinguishes between *what* is being measured (the objectives) and *how* it is being measured (the process). This taxonomy, adapted from recent surveys in the field of LLM agent evaluation, provides a conceptual map for the rest of the framework.3

#### **Evaluation Objectives (The "What")**

This dimension focuses on the specific facets of the agent's performance that are under scrutiny.

* **Agent Behavior:** This is an outcome-oriented, "black-box" view of performance. It answers the question: "Did the agent do what the user wanted?" Key aspects include task completion success rates and the quality of the final output or state change on the blockchain.3  
* **Agent Capabilities:** This is a process-oriented, "white-box" view that examines the agent's underlying competencies. It seeks to understand *how* the agent achieves its goals. For a Solana-native agent, the most critical capability is **tool use**—the ability to correctly select and invoke Solana programs and RPC calls. Other capabilities include planning, reasoning, and context retention over multi-step interactions.3  
* **Reliability & Robustness:** This objective assesses the consistency and resilience of the agent. It measures the agent's ability to perform consistently across similar inputs and its capacity to handle unexpected errors, such as failed transactions or invalid RPC responses, without catastrophic failure.3  
* **Safety & Alignment:** This evaluates the agent's adherence to predefined constraints and ethical guidelines. In the Solana context, this translates to avoiding harmful or wasteful actions, such as sending funds to incorrect addresses, executing transactions that unnecessarily consume high amounts of gas, or interacting with malicious programs.3

#### **Evaluation Process (The "How")**

This dimension describes the methodology and infrastructure used to conduct the evaluation.

* **Interaction Mode:** For maximum reproducibility, the evaluation process will be **static and offline**. The agent will interact with a fixed, predefined set of tasks from a benchmark dataset, rather than with a live human user. This eliminates the variability of human interaction and ensures that every evaluation run is comparable.3  
* **Datasets & Benchmarks:** A custom, domain-specific benchmark, which will be referred to as SolanaBench, is required. General-purpose LLM benchmarks are insufficient as they do not test the agent's ability to interact with Solana-specific primitives. This benchmark will consist of curated test cases, each with a defined initial state, a user prompt, and ground-truth success criteria.2  
* **Metrics & Tooling:** The evaluation will employ a suite of both quantitative and qualitative metrics to cover the objectives defined above. This will be orchestrated by an automated evaluation harness that manages the test execution, data collection, and results aggregation. This tooling is essential for making the evaluation process scalable and repeatable.3

By structuring the framework around this taxonomy, we ensure a holistic assessment that not only measures the agent's final success but also provides deep diagnostic insights into its internal reasoning processes and capabilities, all within a reproducible and verifiable structure.

## **Part II: Architecting the Solana-Gym Evaluation Environment**

This section provides the detailed architectural specification for the SolanaEnv, the custom Gymnasium environment that will serve as the core of the evaluation framework. This environment is designed to be a high-fidelity, hermetic sandbox that accurately simulates interaction with the Solana blockchain while guaranteeing reproducibility. The specification is presented in a language-agnostic manner, focusing on the logic and interfaces that will be implemented in Rust.

### **2.1 The SolanaEnv Class Specification**

The SolanaEnv class will be a subclass of gymnasium.Env and must implement its standard interface. This structure ensures compatibility with the broader ecosystem of agent development and evaluation tools.13

* **\_\_init\_\_(self, config)**: The constructor will initialize the environment. The config object will contain necessary parameters such as the path to the solana-test-validator executable, RPC and WebSocket URLs for the local validator, and paths to any necessary program keypairs or account data files. It will also initialize the action\_space and observation\_space attributes, which define the agent-environment interface.  
* **reset(self, seed=None, options=None)**: This method is critical for establishing a clean and deterministic starting point for each evaluation episode. Its execution flow is as follows:  
  1. It must first call super().reset(seed=seed). This seeds the environment's internal random number generator (self.np\_random) provided by the Gymnasium base class, ensuring any stochasticity within the environment itself is reproducible.14  
  2. It then ensures any existing solana-test-validator process is terminated.  
  3. A new solana-test-validator instance is started. The options dictionary can be used to pass in the path to a specific ledger state snapshot or a set of startup scripts. This allows each test case to begin from a unique and precisely defined on-chain state.  
  4. The method waits for the validator to be fully initialized and ready to accept RPC requests.  
  5. Finally, it queries the initial state of relevant accounts (as defined by the test case) and formats this information into the initial observation dictionary, which is returned to the agent along with an empty info dictionary.  
* **step(self, action: ActType)**: This method is the engine of the environment, processing a single agent action and advancing the state of the world.  
  1. The action received from the agent will be a dictionary representing a tool call (e.g., a specific transaction to execute).  
  2. The environment logic will parse this dictionary, construct the corresponding Solana transaction, sign it (using a pre-configured keypair for the agent's identity), and send it to the local test validator via an RPC call.  
  3. It will then wait for the transaction to be confirmed, with a reasonable timeout.  
  4. Upon confirmation or failure, it will query the transaction's status, logs, and any resulting changes to account states.  
  5. This information is then used to compute the four-tuple return value:  
     * observation: The new state of the world that the agent can perceive.  
     * reward: A scalar value calculated based on the success of the action and its contribution to the overall task goal.  
     * terminated: A boolean flag set to True if the task has been successfully completed or has reached an irrecoverable failure state.  
     * info: A dictionary containing rich diagnostic data not meant for the agent, such as the transaction signature, gas consumed, and raw logs. This data is vital for metrics calculation.  
* **render(self)**: This method provides a human-readable view of the environment's state. The primary implementation will be for the ansi mode.13 When called, it will print a formatted summary to the console, including the current slot, the status of the last transaction, and the balances of key accounts being tracked in the current test case.  
* **close(self)**: This method is responsible for cleanup. It will ensure that the solana-test-validator process is cleanly terminated and any other resources (like RPC connections) are released.

### **2.2 Defining the Action and Observation Spaces**

The action\_space and observation\_space define the formal contract between the agent and the environment. Their careful design is critical for enabling the agent to perceive the state effectively and express its intentions clearly.13

* **action\_space**: The agent's actions are tool calls, which map directly to Solana transactions or RPC queries. A gymnasium.spaces.Dict is the ideal structure for this. The keys of the dictionary will be the names of the available tools. The value for each key will be another Dict space defining the parameters for that tool.  
  * **Example**:  
    Python  
    \# Conceptual representation of the action space  
    action\_space \= spaces.Dict({  
        "tool\_name": spaces.Discrete(3), \# 0: getBalance, 1: transfer, 2: callProgram  
        "parameters": spaces.Dict({  
            "to\_pubkey": spaces.Text(max\_length=44),  
            "amount\_lamports": spaces.Box(low=0, high=1e12, shape=(1,), dtype=np.uint64),  
            "program\_id": spaces.Text(max\_length=44),  
            "instruction\_data": spaces.Text(max\_length=1024) \# e.g., base64 encoded  
        })  
    })

This structure allows the agent to specify which tool it wants to use and provide the necessary arguments. This directly models the function-calling capabilities that are a primary focus of the evaluation.8

* **observation\_space**: The observation is the slice of the world state that the agent is allowed to see to make its next decision. Following best practices for complex environments, this will also be a gymnasium.spaces.Dict.14  
  * **Example**:  
    Python  
    \# Conceptual representation of the observation space  
    observation\_space \= spaces.Dict({  
        "last\_transaction\_status": spaces.Discrete(2), \# 0: Failure, 1: Success  
        "last\_transaction\_error": spaces.Text(max\_length=256),  
        "last\_transaction\_logs": spaces.Text(max\_length=4096),  
        "account\_states": spaces.Dict({  
            \# Dynamically populated with relevant accounts for the task  
            "USER\_WALLET\_PUBKEY": spaces.Box(low=0, high=1e12, shape=(1,), dtype=np.uint64),  
            "PROGRAM\_DATA\_ACCOUNT": spaces.Text(max\_length=8192) \# e.g., base64 encoded data  
        })  
    })

This provides the agent with structured feedback on its previous action and an updated view of the relevant parts of the on-chain state, enabling it to plan its next move.

### **2.3 Ensuring Hermetic Reproducibility: The Local Test Validator**

The single most important component for guaranteeing reproducibility is the exclusive use of a local, ephemeral solana-test-validator instance for all evaluations. Interacting with any shared, public network (including devnet or testnet) would introduce uncontrollable non-determinism from other users' transactions and network latency, making true reproduction of results impossible. The evaluation process must be hermetic—completely self-contained and isolated from external influences.16

The workflow for each test case execution will be as follows:

1. **Setup**: The evaluation harness, before starting a test case, will programmatically start a new solana-test-validator process. This can be done by specifying a command like solana-test-validator \--reset \--ledger /tmp/test-ledger.  
2. **State Loading**: The specific initial on-chain state required by the test case (e.g., accounts with specific balances, deployed programs) will be loaded onto this fresh validator. This can be achieved either by using the \--clone feature to copy accounts from a public network at a specific slot or, more robustly, by executing a setup script that programmatically creates and funds all necessary accounts and deploys programs.  
3. **Execution**: The SolanaEnv instance connects to this isolated validator. The agent then interacts with it for the duration of the test case episode. All transactions occur within this sandbox, with no external effects.  
4. **Teardown**: Once the episode is terminated or truncated, the evaluation harness terminates the solana-test-validator process and cleans up the temporary ledger files.

This strict protocol ensures that every single evaluation run for a given test case starts from the *exact same* blockchain state, down to the last byte. This eliminates an entire class of variables and is a non-negotiable prerequisite for the Solana Foundation or any other third party to be able to independently verify the reported results.

To provide maximum clarity for the implementation team, the following table explicitly maps the abstract components of the Gymnasium API to their concrete counterparts in the Solana ecosystem. This serves as a formal specification to guide the Rust implementation.

| Gymnasium Component | Type | Solana Mapping | Rationale & Example |
| :---- | :---- | :---- | :---- |
| action | Dict | A structured representation of a tool call before serialization into a transaction. | The agent's decision, e.g., {'tool': 'transfer', 'params': {'to': '...', 'amount': 100}}. This dictionary is the action that the step method receives. |
| observation | Dict | The minimal state information required for the agent's next decision, derived from RPC calls like getTransaction and getAccountInfo. | What the agent "sees" post-action, e.g., {'status': 'Success', 'logs':}. |
| reward | float | A scalar value computed from transaction outcomes and progress towards the task goal. | 1.0 if a required transfer succeeded, \-0.1 if it failed, 0.0 for neutral queries. This guides the agent during training and serves as a performance indicator. |
| terminated | bool | A flag indicating if the task's final success or failure conditions have been met, checked via on-chain state assertions. | True once the target account balance is reached or an unrecoverable error (e.g., insufficient funds) occurs. |
| truncated | bool | A flag indicating an external limit was reached, such as a maximum number of steps (transactions) or a time limit. | True if the agent has not completed the task within a predefined limit of, for example, 10 steps. |
| info | Dict | A rich dictionary of diagnostic data for analysis and logging, not visible to the agent. | Contains metadata for metrics calculation, e.g., {'tx\_signature': '...', 'gas\_used': 5000, 'final\_balances': {'from': 900, 'to': 200}}. |

## **Part III: Designing the `reev-benchmarks` Suite**

With the environment's architecture defined, the focus now shifts to the content: the `reev-benchmarks` suite. A benchmark is more than just a set of tests; it is an embodiment of the desired capabilities of the agent. A well-designed benchmark provides a comprehensive measure of performance and serves as a clear target for model improvement. This section details the principles and structure for creating a robust, domain-specific benchmark for Solana-native agents. The benchmark itself should be treated as a first-class project artifact, version-controlled and maintained alongside the agent's source code, as it forms an executable specification of correct and desirable behavior.

### **3.1 A Taxonomy of Solana Agent Capabilities to Test**

To ensure the benchmark has comprehensive coverage, its tasks should be designed to probe a specific set of capabilities. This taxonomy provides a structured approach, moving from simple, atomic skills to complex, integrated behaviors.

* **T1: State Comprehension & Parsing:** This is the agent's ability to perceive the environment accurately. Tasks in this category test whether the agent can issue the correct read-only RPC calls and correctly parse the results.  
  * *Examples:* "What is the SOL balance of wallet ...?", "Is the account ... a token account?", "What is the mint authority for the token ...?".  
* **T2: Tool Selection & Parameterization:** This is the core function-calling capability. Given a natural language prompt, the agent must identify the correct on-chain action (the "tool") and correctly extract and format all necessary parameters for the transaction instruction. This is a critical area of evaluation for any tool-using agent.2  
  * *Examples:* For the prompt "Send 0.5 SOL to Bob," the agent must select the system\_program::transfer tool and correctly parameterize it with the source pubkey, destination pubkey (Bob's address), and the amount in lamports (500000000).  
* **T3: Sequential & Multi-Step Reasoning:** This capability involves executing a sequence of actions where the output of one step is required as the input for a subsequent step. This tests the agent's planning and context-retention abilities.  
  * *Examples:* "Create a new token account for USDC in my wallet, and then send 10 USDC from my primary token account to this new one." This requires a sequence of createAssociatedTokenAccount followed by token::transfer.  
* **T4: Robustness & Error Handling:** This tests the agent's resilience in the face of failure. The environment can be set up to guarantee transaction failures to see how the agent reacts.  
  * *Examples:* "Attempt to transfer 100 SOL from a wallet that only contains 10 SOL." A robust agent should not get stuck in a retry loop; it should recognize the "insufficient funds" error, report the failure, and terminate.  
* **T5: Economic Efficiency & Optimization:** This advanced capability measures the agent's ability to achieve a goal while minimizing resource consumption. On Solana, the primary resource is transaction fees (gas).  
  * *Examples:* "Send 1 SOL to Alice, 2 SOL to Bob, and 3 SOL to Carol." An inefficient agent might create three separate transactions. A more optimized agent would bundle these into a single transaction with multiple transfer instructions, saving significant fees.

### **3.2 Anatomy of a `reev-benchmarks` Test Case**

To enable automated processing by the evaluation harness, every test case in the `reev-benchmarks` suite must adhere to a standardized, machine-readable format. A YAML file per test case is the standard. This structure makes the benchmark explicit and easy to audit.

Each test case file will contain the following key sections:

* **id**: A unique identifier for the test case (e.g., TRANSFER-SIMPLE-001).  
* **description**: A human-readable string explaining the task's objective.  
* **tags**: A list of tags for categorization (e.g., \[token-program, t1, t2\]) to allow for running subsets of the benchmark.  
* **initial\_state**: A declarative definition of the on-chain state required at the beginning of the test. This includes a list of accounts to create, their owners, lamport balances, and any initial data (e.g., for token accounts or program state). This section is used by the setup script to prepare the solana-test-validator instance.  
* **prompt**: The natural language prompt that is given to the agent as its instruction.  
* **ground\_truth**: This section contains the objective criteria for judging the agent's performance on this test case. It is the core of the evaluation logic.  
  * **final\_state\_assertions**: A list of conditions that must be true on the blockchain after the agent has finished. Each assertion specifies a type (e.g., SolBalance, TokenAccountBalance), an account to check, and an expected value or condition (e.g., equals, greater\_than). The overall Task Success Rate is determined by the pass/fail status of these assertions.  
  * **expected\_tool\_calls**: An ordered list of the ideal tool calls the agent should make to solve the task efficiently and correctly. Each entry includes the tool\_name and can optionally include assertions about the parameters. This data is used to calculate metrics like Tool Selection Accuracy and serves as a reference for the LLM-as-a-Judge evaluation.5

Below is an illustrative example of a test case in YAML format:

YAML

id: SWAP-001  
description: "Successfully swaps 100 USDC for SOL on a mock DEX."  
tags: \[defi, multi-step, t2, t3\]  
initial\_state:  
  \# Defines accounts and their data to load into the test validator  
  \- pubkey: "USER\_WALLET\_PUBKEY"  
    lamports: 1000000000 \# 1 SOL  
    owner: "11111111111111111111111111111111"  
  \- pubkey: "USER\_USDC\_ATA"  
    owner: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"  
    data: "..." \# Base64 encoded token account data for 100 USDC  
  \- pubkey: "MOCK\_DEX\_PROGRAM\_ID"  
    is\_executable: true  
    data\_from\_file: "./mock\_dex.so"  
prompt: "I want to swap my 100 USDC for as much SOL as possible."  
ground\_truth:  
  final\_state\_assertions:  
    \# Conditions to check on the blockchain state post-execution  
    \- type: "TokenAccountBalance"  
      pubkey: "USER\_USDC\_ATA"  
      expected: 0  
    \- type: "SolBalanceChange"  
      pubkey: "USER\_WALLET\_PUBKEY"  
      expected\_change\_gte: 500000000 \# Expected to gain at least 0.5 SOL, accounting for fees  
  expected\_tool\_calls:  
    \# An ordered list of tools the agent should ideally call  
    \- tool\_name: "get\_swap\_quote"  
      params: { input\_token: "USDC", output\_token: "SOL", amount: 100000000 }  
    \- tool\_name: "execute\_swap"

### **3.3 Curating the Benchmark: From Unit Tests to Integration Tests**

Building a comprehensive benchmark suite is an iterative process. The strategy should mirror standard software development practices, starting with simple cases and building up to complex scenarios.

* **Phase 1: Unit Test Cases:** These are the simplest tasks, designed to test one capability in isolation, primarily T1 (State Comprehension) and T2 (Tool Selection). They should involve a single tool call.  
  * *Example Task:* "What is the current slot height?"  
  * *Purpose:* To validate the agent's basic understanding of the available tools and its ability to handle simple queries.  
* **Phase 2: Integration Test Cases:** These tasks require multiple steps and test the integration of several capabilities, particularly T3 (Sequential Reasoning).  
  * *Example Task:* "Check if I have a token account for the BONK token. If not, create one. Then, report the final balance."  
  * *Purpose:* To assess the agent's planning, memory, and ability to chain actions together.  
* **Phase 3: Adversarial & Edge Cases:** These tasks are designed to probe the agent's robustness and safety (T4 and T5). They include scenarios that should result in predictable failures or require optimization.  
  * *Example Task:* "Send 1 SOL to address ..., but use the lowest possible transaction fee." (Tests T5)  
  * *Example Task:* "Call a non-existent instruction on the Memo program." (Tests T4)  
  * *Purpose:* To identify failure modes and ensure the agent behaves safely and predictably when things go wrong.

The initial set of test cases should be curated manually by domain experts to ensure quality and relevance. Once a solid base of 50-100 test cases is established, this set can be expanded using **synthetic data generation**. This involves using a powerful LLM (like GPT-4) to generate variations of existing prompts or create entirely new scenarios based on a template.15 This is a cost-effective way to increase the size and diversity of the benchmark, helping to prevent the agent from "overfitting" to a small set of hand-crafted examples.

## **Part IV: A Multi-Faceted Metrics and Evaluation Harness**

A benchmark is only as useful as the metrics used to score performance against it. A single metric is insufficient to capture the nuanced behavior of an autonomous agent. Therefore, this framework employs a multi-faceted approach, combining objective, quantitative metrics with nuanced, qualitative assessments. These are all managed and executed by a central "Evaluation Harness," an automated system that orchestrates the entire process from start to finish.

### **4.1 Suite of Quantitative Performance Metrics**

These metrics are automatically calculated by the evaluation harness based on the execution trace and the ground truth defined in each test case. They provide the objective, reproducible core of the performance assessment.

* **Task Success Rate (TSR):** This is the primary, top-level metric of agent behavior. It is the percentage of test cases in the benchmark for which all final\_state\_assertions pass at the end of the episode. A high TSR indicates that the agent is effective at achieving its goals.3  
  * *Calculation:* TSR=Total number of test casesNumber of test cases with all assertions passed​  
* **Tool Selection Accuracy (TSA):** This metric evaluates a core agent capability: choosing the correct tool for the job. It is calculated by comparing the sequence of tools called by the agent against the expected\_tool\_calls sequence in the ground truth. It can be measured using standard classification metrics like precision, recall, and F1-score over the sequence of calls.5  
  * *Example:* If the ground truth expects \`\` and the agent calls \[A, C\], the precision is 1/2, recall is 1/2, and F1 is 0.5.  
* **Parameterization Accuracy (PA):** This metric drills deeper than TSA. For the tool calls that were correctly selected, it measures whether the agent provided the correct parameters. This is crucial, as calling the right tool with the wrong arguments will lead to failure. Accuracy can be calculated per-parameter or as an exact match for the entire parameter set.  
  * *Example:* For a transfer call, PA would check if the to\_pubkey and amount\_lamports match the ground truth.  
* **Gas Consumption Efficiency (GCE):** A critical metric for any on-chain application, GCE measures the total computational units (and thus, transaction fees) consumed by the agent to complete a task successfully. This can be reported as an absolute value or normalized against a pre-calculated "optimal" gas usage for that task, yielding a score where lower is better.  
  * *Calculation:* GCE=∑i=1N​gas\_used(transactioni​), where N is the number of transactions in a successful episode.  
* **Execution Latency (EL):** This measures the wall-clock time from the start of the episode until the terminated flag is set to True. It captures the overall speed of the agent, including both its internal "thinking" time and the time spent waiting for transaction confirmations.

### **4.2 Qualitative Assessment via LLM-as-a-Judge**

Quantitative metrics can determine *if* an agent succeeded, but they often fail to capture *how* it succeeded or *why* it failed. For instance, an agent might complete a task but take a convoluted, illogical, or inefficient path. To assess these qualitative aspects of reasoning and planning at scale, the framework incorporates the "LLM-as-a-Judge" pattern.18

This process uses a powerful, state-of-the-art LLM (e.g., GPT-4o, Claude 3 Opus) as an impartial evaluator to score the agent's execution trace. This provides a scalable proxy for human evaluation.20

The process is as follows:

1. **Define Evaluation Criteria:** A detailed rubric is created to guide the judge LLM. This rubric breaks down the assessment into specific, well-defined criteria.  
   * **Reasoning Coherence**: Does the agent's plan follow a logical, step-by-step process? Is its internal monologue (if available) consistent with its actions?  
   * **Plan Efficiency**: Did the agent choose the most direct and resource-efficient path to the solution, or was its plan unnecessarily complex?  
   * **Adaptability**: If an error occurred, did the agent identify the root cause correctly and take a sensible corrective action?  
2. **Prepare the Evaluation Prompt:** For each completed test case, the evaluation harness automatically constructs a prompt for the judge LLM. This prompt includes:  
   * The original user prompt from the test case.  
   * The full, unabridged execution trace, including the agent's internal thoughts and the sequence of tool calls, parameters, and results.  
   * The detailed evaluation rubric and a request for a score (e.g., on a scale of 1-5) and a written justification for each criterion.  
3. **Score Generation and Aggregation:** The harness sends this prompt to the judge LLM's API and parses the structured response. The resulting scores and rationales are added to the test case results.

This hybrid approach, combining hard quantitative metrics with qualitative LLM-based scores, provides a holistic view of agent performance. It aligns with best practices in modern LLM evaluation, where automated unit tests are complemented by more nuanced, semantic assessments.21

### **4.3 The Evaluation Runner: Orchestrating the Process**

The Evaluation Runner is the master script or application that automates the entire evaluation workflow. It is the practical implementation that connects the SolanaEnv, the SolanaBench benchmark, and the metrics calculation modules. Its high-level logic is crucial for ensuring the process is repeatable and easy to execute.

The runner's operational flow can be described with the following pseudo-code:

function run\_evaluation(agent\_model, benchmark\_path, output\_path):  
  // Load the entire benchmark suite  
  benchmark \= load\_benchmark\_from\_path(benchmark\_path)  
  all\_results \=

  for each test\_case in benchmark:  
    // 1\. Setup the hermetic environment for this specific test case  
    env \= initialize\_solana\_env(test\_case.initial\_state)  
      
    // 2\. Reset the environment with a deterministic seed  
    observation, info \= env.reset(seed=test\_case.seed)  
      
    // 3\. Initialize data structures for tracing and logging  
    execution\_trace \= Trace()  
    is\_done \= False  
      
    // 4\. Run the agent-environment interaction loop  
    while not is\_done:  
      // Agent makes a decision based on the current observation  
      action, thought\_process \= agent\_model.get\_action(observation)  
        
      // Record the agent's thought and intended action  
      execution\_trace.add\_step(thought=thought\_process, action=action)  
        
      // The environment processes the action and returns the outcome  
      observation, reward, terminated, truncated, info \= env.step(action)  
        
      // Record the environment's response  
      execution\_trace.add\_step(observation=observation, info=info)  
        
      is\_done \= terminated or truncated  
        
    // 5\. Calculate all metrics for the completed episode  
    quantitative\_scores \= calculate\_quantitative\_metrics(env, execution\_trace, test\_case.ground\_truth)  
    qualitative\_scores \= run\_llm\_as\_judge(execution\_trace, test\_case.prompt)  
      
    // 6\. Aggregate results for this test case  
    test\_result \= {  
      "test\_case\_id": test\_case.id,  
      "trace": execution\_trace.to\_json(),  
      "quantitative": quantitative\_scores,  
      "qualitative": qualitative\_scores  
    }  
    all\_results.append(test\_result)  
      
    // 7\. Clean up the environment  
    env.close()

  // 8\. Generate and save the final summary report  
  generate\_summary\_report(all\_results, output\_path)  
  return all\_results

This orchestrated process, conceptually similar to frameworks like OpenAI Evals 17, ensures that every test case is run in a clean, isolated environment and that all relevant data is captured for a comprehensive, multi-faceted analysis.

## **Part V: Visualizing Execution Traces as Action Graphs**

A critical requirement for any complex AI system is interpretability. For an autonomous agent performing actions on a blockchain, stakeholders need to understand not just *what* the agent did, but *why* it did it. A raw log file or a JSON object is insufficient for intuitive understanding. The visualization of the agent's execution trace as a structured graph or tree is therefore not a cosmetic feature, but an essential tool for debugging, analysis, and building trust.22 This section details the process of capturing the necessary data and rendering it into a human-readable ASCII tree format, providing a form of "Explainable AI" (XAI) for agentic systems.

### **5.1 Instrumentation and Trace Capture**

Before an execution can be visualized, it must be meticulously recorded. This process, known as **tracing** or **instrumentation**, involves capturing every significant event in the agent's interaction loop in a structured format.23 The Evaluation Runner described in the previous section is responsible for this data capture.

The trace should be stored as a hierarchical data structure, such as a tree, where each node represents a distinct step in the agent's reasoning process. A JSON object is a suitable format for serializing this trace.

A node in the trace tree could have the following attributes:

* **node\_type**: The type of event (e.g., PLAN, TOOL\_CALL, TOOL\_RESULT, OBSERVATION).  
* **content**: A dictionary containing the data for that event.  
  * For a PLAN node, this might be the agent's textual "thought process."  
  * For a TOOL\_CALL node, this would include the tool\_name and the parameters.  
  * For a TOOL\_RESULT node, this would contain the data returned by the tool, such as a transaction signature or an error message.  
* **children**: A list of child nodes, allowing for the representation of sequential or nested operations.

Here is an example of a JSON trace for a multi-step task, which captures the agent's plan and subsequent tool interactions. This structured log is the raw material for both the LLM-as-a-Judge evaluation and the visualization rendering.25

JSON

{  
  "prompt": "Create a USDC token account and then send 10 USDC to it.",  
  "execution\_tree": {  
    "node\_type": "PLAN",  
    "content": {  
      "thought": "The user wants to create a new USDC account and then fund it. I need to perform two steps. First, call the tool to create an associated token account. Second, use the new account's address to call the token transfer tool."  
    },  
    "children":  
      },  
      {  
        "node\_type": "TOOL\_CALL",  
        "content": {  
          "tool\_name": "token\_transfer",  
          "parameters": {  
            "source\_pubkey": "PRIMARY\_USDC\_ATA",  
            "destination\_pubkey": "NEW\_USDC\_ATA\_PUBKEY",  
            "amount": 10000000  
          }  
        },  
        "children":  
      }  
    \]  
  }  
}

### **5.2 From Hierarchical Trace to ASCII Tree**

With the execution captured in a structured hierarchical format, it can be rendered into the requested ASCII tree. This provides an intuitive, inline-friendly visualization that clearly shows the agent's chain of thought and the sequence of actions it took. The rendering process involves a recursive traversal of the trace tree.

The algorithm, presented here in pseudo-code, can be implemented in any language, including Rust. It demonstrates the core logic of managing indentation and connector characters (|, \+--, ) to create the tree structure. This approach is inspired by the internal logic of ASCII tree generation libraries.27

function render\_trace\_to\_ascii(trace\_json):  
  root\_node \= trace\_json.execution\_tree  
  // Start the recursive rendering from the root node  
  return recursive\_render(root\_node, prefix="")

function recursive\_render(node, prefix):  
  output\_string \= ""  
    
  // Format the content of the current node for display  
  node\_info \= format\_node\_content(node.node\_type, node.content)  
  output\_string \+= prefix \+ "+-- " \+ node\_info \+ "\\n"  
    
  if node has children:  
    num\_children \= length(node.children)  
    for i from 0 to num\_children \- 1:  
      child \= node.children\[i\]  
        
      // Determine the prefix for the child node  
      // The last child gets a different connector  
      if i \== num\_children \- 1:  
        child\_prefix \= prefix \+ "    "  
      else:  
        child\_prefix \= prefix \+ "| "  
          
      // Recursively call the render function for the child  
      output\_string \+= recursive\_render(child, child\_prefix)  
        
  return output\_string

function format\_node\_content(type, content):  
  // Helper function to create a concise one-line summary of the node  
  if type \== "PLAN":  
    return "PLAN: " \+ content.thought.substring(0, 80\) \+ "..."  
  if type \== "TOOL\_CALL":  
    params\_str \= format\_parameters(content.parameters)  
    return "TOOL\_CALL: " \+ content.tool\_name \+ "(" \+ params\_str \+ ")"  
  if type \== "TOOL\_RESULT":  
    return "RESULT: " \+ format\_result(content)  
  //... other node types

Applying this algorithm to the example JSON trace from the previous section would produce the following output. This visualization immediately clarifies the agent's two-step plan and the success of each action, providing a powerful diagnostic tool that is far more intuitive than reading the raw JSON.

\+-- PLAN: The user wants to create a new USDC account and then fund it. I need to per...  
    \+-- TOOL\_CALL: create\_associated\_token\_account(owner\_pubkey=USER..., mint\_pubkey=USDC...)

| \+-- RESULT: status=Success, new\_account\_pubkey=NEW\_..., tx\_signature=...  
    \+-- TOOL\_CALL: token\_transfer(source\_pubkey=PRIMARY..., destination\_pubkey=NEW\_..., amount=10000000)  
        \+-- RESULT: status=Success, tx\_signature=...

This ASCII trace is the final output that transforms the evaluation from a simple pass/fail judgment into a rich, explainable process. When a test case fails, this trace will be the primary artifact for debugging, allowing developers to instantly pinpoint the exact moment the agent's logic deviated from the desired path. This level of transparency is essential for iterating on complex agentic systems and for building the necessary trust for their deployment in real-world, high-stakes environments like the Solana ecosystem.

## **Part VI: Porting the Gymnasium Framework to Rust for LLM Agent Evaluation**

While the Gymnasium API provides a language-agnostic conceptual standard, a native Rust implementation is desirable for performance, type safety, and seamless integration with the broader Rust and Solana ecosystems. This section outlines a step-by-step plan for translating the Python-based Gymnasium concepts into an idiomatic Rust library for evaluating the Solana LLM agent.

### **6.1 Phase 1: Defining Core Traits and Structs**

The first step is to establish a Rust-native equivalent of the Gymnasium Env class and its associated data structures. This is best achieved using Rust's trait system to define the interface and structs for the data.

1. **Define the GymEnv Trait:** This trait will be the direct analog to the gymnasium.Env base class, defining the contract that any environment must follow.13  
   Rust  
   // Represents the output of a step  
   pub struct Step\<Obs\> {  
       pub observation: Obs,  
       pub reward: f32,  
       pub terminated: bool,  
       pub truncated: bool,  
       pub info: serde\_json::Value, // For flexible diagnostic data  
   }

   // The core environment trait  
   pub trait GymEnv {  
       type Action;  
       type Observation;

       // Resets the environment, returning the initial observation  
       fn reset(&mut self, seed: Option\<u64\>, options: Option\<serde\_json::Value\>) \-\> anyhow::Result\<Self::Observation\>;

       // Processes an action and returns the outcome  
       fn step(&mut self, action: Self::Action) \-\> anyhow::Result\<Step\<Self::Observation\>\>;

       // Renders a view of the environment  
       fn render(&self);

       // Cleans up resources  
       fn close(&mut self);  
   }

2. **Define Action and Observation Structs:** These structs will represent the data exchanged between the agent and the environment. Using serde for serialization will make it easy to pass this data to and from the LLM, which will likely communicate via JSON.  
   Rust  
   use serde::{Deserialize, Serialize};  
   use std::collections::HashMap;

   \#  
   pub struct AgentAction {  
       pub tool\_name: String,  
       pub parameters: HashMap\<String, serde\_json::Value\>,  
   }

   \#  
   pub struct AgentObservation {  
       pub last\_transaction\_status: String, // "Success" or "Failure"  
       pub last\_transaction\_error: Option\<String\>,  
       pub last\_transaction\_logs: Vec\<String\>,  
       pub account\_states: HashMap\<String, serde\_json::Value\>,  
   }

### **6.2 Phase 2: Implementing the SolanaEnv Struct**

With the interface defined, the next phase is to create the concrete implementation for the Solana environment.

1. **Create the SolanaEnv Struct:** This struct will hold the state needed to manage the environment, including the test validator process and the RPC client.  
   Rust  
   use solana\_client::rpc\_client::RpcClient;  
   use solana\_sdk::signer::keypair::Keypair;  
   use std::process::Child;

   pub struct SolanaEnv {  
       validator\_process: Option\<Child\>,  
       rpc\_client: RpcClient,  
       agent\_keypair: Keypair,  
       //... other configuration fields  
   }

2. **Implement the GymEnv Trait for SolanaEnv:** This involves writing the logic for each method defined in the trait.  
   * **reset:** This function will be responsible for killing any existing validator process, starting a new solana-test-validator instance using std::process::Command, waiting for it to become responsive, and then fetching the initial on-chain state to return the first AgentObservation.  
   * **step:** This is the core logic loop. It will take an AgentAction, parse it, use the solana-sdk to construct the appropriate transaction, sign it with agent\_keypair, and send it using the rpc\_client. After waiting for confirmation, it will query the transaction status and logs to build and return the Step\<AgentObservation\> result.  
   * **render:** This will print a formatted summary of the current state (e.g., last transaction signature, key account balances) to the console.  
   * **close:** This method will ensure the validator\_process is properly terminated using its kill() method.

### **6.3 Phase 3: Interfacing with the LLM Agent**

The Rust environment needs a way to communicate with the LLM to get actions and provide observations. Since the LLM is an external component, a clear API boundary is required.

1. **Define the Communication Protocol:** An HTTP-based REST API is a straightforward choice. The evaluation harness will host a simple server (using a framework like axum or actix-web) that the LLM can call, or it will act as a client calling an external LLM API.  
2. Structure the Agent Loop: The main evaluation loop in Rust will orchestrate the interaction:  
   a. Serialize the current AgentObservation to JSON.  
   b. Send an HTTP POST request to the LLM's endpoint with the observation.  
   c. Await the response, which should be a JSON representation of the AgentAction.  
   d. Deserialize the JSON into an AgentAction struct.  
   e. Pass the action to solana\_env.step().  
   f. Repeat the loop with the new observation.

### **6.4 Phase 4: Building the Evaluation Harness in Rust**

The final piece is the top-level application that runs the entire benchmark.

1. **Benchmark Loading:** The harness will start by reading and parsing the SolanaBench YAML or JSON files into a vector of Rust structs. The serde\_yaml or serde\_json crates are ideal for this.  
2. **Test Case Iteration:** The harness will loop through each parsed test case.  
3. Environment Orchestration: For each test case, it will:  
   a. Instantiate a new SolanaEnv, configured with the initial\_state from the test case.  
   b. Call solana\_env.reset() to start the validator and get the initial state.  
   c. Run the agent-environment interaction loop (as described in Phase 3\) until the terminated or truncated flag is true.  
   d. During the loop, record every action, observation, and piece of info into a trace data structure.  
4. **Metrics Calculation and Reporting:** After each test case completes, the harness will use the recorded trace and the ground\_truth from the benchmark file to calculate all the quantitative metrics (TSR, TSA, etc.). The trace will also be formatted for the LLM-as-a-Judge evaluation and rendered as an ASCII tree.  
5. **Final Report Generation:** Once all test cases have been run, the harness will aggregate the results into a final summary report and save it to a file.

By following this phased approach, the robust, reproducible evaluation framework defined in this document can be systematically translated into a high-performance, type-safe Rust application, ready to rigorously assess the capabilities of Solana-native LLM agents.

## **Conclusion**

The proposed framework provides a comprehensive, rigorous, and reproducible methodology for the evaluation of custom, tool-using LLM agents operating on the Solana blockchain. By grounding the architecture in the industry-standard Gymnasium API, it ensures a structured and familiar approach to modeling agent-environment interactions. The insistence on a hermetic evaluation environment, powered by an ephemeral solana-test-validator, is the cornerstone of reproducibility, guaranteeing that results can be independently verified by any third party.

The multi-faceted evaluation process, combining objective quantitative metrics with qualitative LLM-as-a-Judge assessments, offers a holistic view of agent performance. It moves beyond simple task success to analyze the quality of the agent's reasoning, the correctness of its tool use, and its efficiency in a resource-constrained environment. The SolanaBench benchmark, designed as a living, version-controlled specification, provides a clear and evolving target for agent development and a robust suite for regression testing.

Finally, the integration of detailed execution tracing and visualization in the form of ASCII action graphs elevates the framework from a mere testing utility to a powerful diagnostic and explainability tool. This transparency is critical for debugging complex agentic behavior and for building the trust necessary for deploying autonomous systems in the decentralized world.

For the implementation team, this document serves as a detailed architectural blueprint. By adhering to these principles—a standardized environment API, hermetic execution, a multi-faceted metrics suite, and transparent tracing—the resulting evaluation framework will not only meet the user's immediate needs but will also align with the best practices of the broader AI research community and satisfy the stringent requirements for verifiability expected within the Solana ecosystem.

#### **Works cited**

1. Exploring LLM evaluations and benchmarking | genai-research – Weights & Biases \- Wandb, accessed September 11, 2025, [https://wandb.ai/onlineinference/genai-research/reports/Exploring-LLM-evaluations-and-benchmarking--VmlldzoxMzk0OTI0OA](https://wandb.ai/onlineinference/genai-research/reports/Exploring-LLM-evaluations-and-benchmarking--VmlldzoxMzk0OTI0OA)  
2. 20 LLM evaluation benchmarks and how they work \- Evidently AI, accessed September 11, 2025, [https://www.evidentlyai.com/llm-guide/llm-benchmarks](https://www.evidentlyai.com/llm-guide/llm-benchmarks)  
3. arxiv.org, accessed September 11, 2025, [https://arxiv.org/html/2507.21504v1](https://arxiv.org/html/2507.21504v1)  
4. accessed January 1, 1970, [https'//solana-foundation.github.io/solana-gym-env/](http://docs.google.com/https'//solana-foundation.github.io/solana-gym-env/)  
5. Navigating the Maze of LLM Evaluation: A Guide to Benchmarks, RAG, and Agent Assessment | by Yuji Isobe | Medium, accessed September 11, 2025, [https://medium.com/@yujiisobe/navigating-the-maze-of-llm-evaluation-a-guide-to-benchmarks-rag-and-agent-assessment-fb7aef299e66](https://medium.com/@yujiisobe/navigating-the-maze-of-llm-evaluation-a-guide-to-benchmarks-rag-and-agent-assessment-fb7aef299e66)  
6. Evaluation and Benchmarking of LLM Agents: A Survey \- arXiv, accessed September 11, 2025, [https://arxiv.org/pdf/2507.21504](https://arxiv.org/pdf/2507.21504)  
7. A Streamlined Framework for Enhancing LLM Reasoning with Agentic Tools \- arXiv, accessed September 11, 2025, [https://arxiv.org/html/2502.04644v2](https://arxiv.org/html/2502.04644v2)  
8. Survey on Evaluation of LLM-based Agents \- arXiv, accessed September 11, 2025, [https://arxiv.org/html/2503.16416v1](https://arxiv.org/html/2503.16416v1)  
9. Solana Documentation, accessed September 11, 2025, [https://solana.com/docs](https://solana.com/docs)  
10. Agentic LLM Framework for Adaptive Decision Discourse \- arXiv, accessed September 11, 2025, [https://arxiv.org/html/2502.10978v1](https://arxiv.org/html/2502.10978v1)  
11. Gym Documentation, accessed September 11, 2025, [https://gymlibrary.ml/](https://gymlibrary.ml/)  
12. Gymnasium Documentation, accessed September 11, 2025, [https://gymnasium.farama.org/](https://gymnasium.farama.org/)  
13. Env \- Gymnasium Documentation, accessed September 11, 2025, [https://gymnasium.farama.org/api/env/](https://gymnasium.farama.org/api/env/)  
14. Make your own custom environment \- Gym Documentation, accessed September 11, 2025, [https://www.gymlibrary.dev/content/environment\_creation/](https://www.gymlibrary.dev/content/environment_creation/)  
15. Evaluating LLM Systems: Essential Metrics, Benchmarks, and Best Practices \- Confident AI, accessed September 11, 2025, [https://www.confident-ai.com/blog/evaluating-llm-systems-metrics-benchmarks-and-best-practices](https://www.confident-ai.com/blog/evaluating-llm-systems-metrics-benchmarks-and-best-practices)  
16. Science of Agent Evaluation: SAgE Research Group, accessed September 11, 2025, [https://sage.cs.princeton.edu/](https://sage.cs.princeton.edu/)  
17. openai/evals: Evals is a framework for evaluating LLMs and ... \- GitHub, accessed September 11, 2025, [https://github.com/openai/evals](https://github.com/openai/evals)  
18. \[2508.05508\] Auto-Eval Judge: Towards a General Agentic Framework for Task Completion Evaluation \- arXiv, accessed September 11, 2025, [https://arxiv.org/abs/2508.05508](https://arxiv.org/abs/2508.05508)  
19. LLM-as-a-judge: a complete guide to using LLMs for evaluations \- Evidently AI, accessed September 11, 2025, [https://www.evidentlyai.com/llm-guide/llm-as-a-judge](https://www.evidentlyai.com/llm-guide/llm-as-a-judge)  
20. When AIs Judge AIs: The Rise of Agent-as-a-Judge Evaluation for LLMs \- arXiv, accessed September 11, 2025, [https://arxiv.org/html/2508.02994v1](https://arxiv.org/html/2508.02994v1)  
21. \\ours: Evaluating LLM Agent's Ability on Reproducing Language Modeling Research \- arXiv, accessed September 11, 2025, [https://arxiv.org/html/2506.17335v1](https://arxiv.org/html/2506.17335v1)  
22. AI traces are worth a thousand logs \- YouTube, accessed September 11, 2025, [https://www.youtube.com/watch?v=yC3XS85CPaQ](https://www.youtube.com/watch?v=yC3XS85CPaQ)  
23. LLM Tracing Explained: Definition & Best Practices | Generative AI Collaboration Platform, accessed September 11, 2025, [https://orq.ai/blog/llm-tracing](https://orq.ai/blog/llm-tracing)  
24. LLM Observability & Application Tracing (open source) \- Langfuse, accessed September 11, 2025, [https://langfuse.com/docs/tracing](https://langfuse.com/docs/tracing)  
25. How to Visually Debug Multi AI-Agent Flows \- DEV Community, accessed September 11, 2025, [https://dev.to/rish\_poddar/how-to-visually-debug-multi-ai-agent-flows-310p](https://dev.to/rish_poddar/how-to-visually-debug-multi-ai-agent-flows-310p)  
26. Evaluating LLM Tool Calls with Laminar, accessed September 11, 2025, [https://docs.lmnr.ai/guides/evaluating-tool-calls](https://docs.lmnr.ai/guides/evaluating-tool-calls)  
27. taeefnajib/pygentree: A Python package to generate ASCII tree representation of directory structures. \- GitHub, accessed September 11, 2025, [https://github.com/taeefnajib/pygentree](https://github.com/taeefnajib/pygentree)  
28. asciitree · PyPI, accessed September 11, 2025, [https://pypi.org/project/asciitree/](https://pypi.org/project/asciitree/)  
29. Build a Python Directory Tree Generator for the Command Line, accessed September 11, 2025, [https://realpython.com/directory-tree-generator-python/](https://realpython.com/directory-tree-generator-python/)

