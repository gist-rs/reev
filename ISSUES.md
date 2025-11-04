# Issues

## Issue #15: GLM-4.6 Model Incorrectly Routed to ZAI Client Instead of OpenAI Client ❌ **NEW**
### 🎯 **Problem Statement**
The `glm-4.6` model should use OpenAI-compatible format with ZAI endpoint, but is being incorrectly routed to ZAI-specific client, causing "Invalid API parameter" errors.

#### ❌ **Current Broken Behavior**
For prompt `"use my 50% sol to multiply usdc 1.5x on jup"` with `glm-4.6`:
```bash
# Expected: glm-4.6 -> OpenAI client -> ZAI endpoint /api/paas/v4
# Actual: glm-4.6 -> ZAI client -> ZAI endpoint /api/coding/paas/v4
Error: "ZAI API error 400 Bad Request: Invalid API parameter"
```

#### ✅ **Expected Behavior**
```bash
glm-4.6 -> OpenAI client -> https://api.z.ai/api/paas/v4 -> success
glm-4.6-coding -> ZAI client -> https://api.z.ai/api/coding/paas/v4 -> success
```

### 📋 **Root Cause Analysis**
1. **Routing Logic Issue**: Multiple routing layers causing confusion
   - API layer (`dynamic_flows/mod.rs`) has separate routing from main dispatcher (`run.rs`)
   - `glm-4.6` being routed to ZAI client instead of OpenAI client
   - `glm-4.6-coding` correctly using ZAI client

2. **Model-Specific API Endpoints**:
   - `glm-4.6` should use: `https://api.z.ai/api/paas/v4` (OpenAI compatible)
   - `glm-4.6-coding` should use: `https://api.z.ai/api/coding/paas/v4` (ZAI specific)

3. **Architecture Confusion**:
   - `openai.rs` handles OpenAI-compatible clients (including glm-4.6)
   - `zai_agent.rs` handles ZAI-specific clients (glm-4.6-coding)
   - Both models currently going to `zai_agent.rs`

### 🛠️ **Solutions Required**

#### **Solution 1**: Fix API Layer Routing
Update the hardcoded routing in `execute_real_agent_for_flow_plan()` to correctly route `glm-4.6` to OpenAI client.

#### **Solution 2**: Clean Up Redundant Routing
Remove duplicate routing logic to prevent future confusion.

### 📊 **Test Cases**
#### **Test 1**: GLM-4.6 Model Routing
```bash
curl -X POST http://localhost:3001/api/v1/benchmarks/execute-direct \
  -H "Content-Type: application/json" \
  -d '{"prompt": "use my 50% sol to multiply usdc 1.5x on jup", "wallet": "test_wallet", "agent": "glm-4.6", "shared_surfpool": false}'
```

**Expected**: Routes to OpenAI client with ZAI endpoint
**Actual**: Routes to ZAI client with coding endpoint

#### **Test 2**: GLM-4.6-coding Model Routing  
```bash
curl -X POST http://localhost:3001/api/v1/benchmarks/execute-direct \
  -H "Content-Type: application/json" \
  -d '{"prompt": "code a smart contract", "wallet": "test_wallet", "agent": "glm-4.6-coding", "shared_surfpool": false}'
```

**Expected**: Routes to ZAI client with coding endpoint  
**Actual**: Routes to ZAI client with coding endpoint (CORRECT)

### 🧪 **Validation Steps**
1. Test both model routing paths work correctly
2. Verify OpenAI client uses correct API endpoint
3. Verify ZAI client uses correct coding endpoint
4. Confirm no more "Invalid API parameter" errors

### 📈 **Impact Assessment**
**Critical**: Core agent routing broken for primary GLM model
**User Impact**: All GLM-4.6 requests fail with API errors
**System Impact**: Blocks all GLM-4.6 benchmark execution

### 🔗 **Related Issues**
- Issue #14 ✅ **RESOLVED**: Mock data removed from production
- Issue #13 ✅ **RESOLVED**: Enhanced transaction visualization working
- Issue #12 ✅ **RESOLVED**: API returns tool calls data

### 🗓️ **Resolution Timeline**
**Priority**: Critical - Blocks all GLM-4.6 model usage
**Estimated**: 1-2 hours for routing logic fix

---