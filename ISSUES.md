# Issues

## Issue #21: ZAI API 400 Bad Request Errors - ACTIVE 🔴

### **Problem Summary**
GLM agents (glm-4.6 and glm-4.6-coding) are encountering ZAI API 400 Bad Request errors with "Invalid API parameter, please check documentation" messages, preventing successful tool execution despite proper API configuration.

### **Root Cause Analysis**
**API Parameter Format Issue**: The ZAI API is rejecting requests due to malformed parameter structure or missing required fields in the tool call JSON sent to the API.

**Error Pattern**:
```
"ProviderError: ZAI API error 400 Bad Request: {\"error\":{\"code\":\"1210\",\"message\":\"Invalid API parameter, please check the documentation.\"}}"
```

**Current Status**:
- ✅ API endpoints correctly constructed (Issue #18 resolved)
- ✅ Agent routing working (glm-4.6 → OpenAI, glm-4.6-coding → ZAI)
- ✅ Dynamic flow orchestration operational
- ❌ ZAI API parameter formatting causing 400 errors

### **Evidence from Current Execution**
```bash
# Test execution showing the error
curl -X POST http://localhost:3001/api/v1/benchmarks/execute-direct \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "use my 50% sol to multiply usdc 1.5x on jup",
    "wallet": "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM",
    "agent": "glm-4.6-coding",
    "shared_surfpool": false
  }'

# Response shows 400 Bad Request errors
{
  "tool_calls": [
    {
      "tool_name": "account_balance",
      "success": false,
      "error": "ProviderError: ZAI API error 400 Bad Request..."
    },
    {
      "tool_name": "jupiter_swap", 
      "success": false,
      "error": "ProviderError: ZAI API error 400 Bad Request..."
    }
  ]
}
```

### **Investigation Required**
1. **Tool Call JSON Structure**: Verify tool parameters are properly formatted for ZAI API
2. **API Request Headers**: Check Content-Type and Authorization headers
3. **Parameter Encoding**: Ensure JSON serialization matches ZAI API expectations
4. **Tool Definition**: Validate tool schemas align with ZAI API requirements

### **Debug Commands**
```bash
# Monitor detailed agent execution logs
tail -f api_server_*.log | grep -E "(ZAI|tool_call|400)"

# Test simple account_balance tool call
curl -X POST http://localhost:3001/api/v1/benchmarks/execute-direct \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "account_balance",
    "wallet": "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM",
    "agent": "glm-4.6-coding",
    "shared_surfpool": false
  }'

# Check OTEL traces for parameter details
tail -f logs/sessions/enhanced_otel_orchestrator-flow-*.jsonl
```

### **Status**: ACTIVE 🔴
- Dynamic flow architecture working correctly
- Tool orchestration generating proper step sequences
- ZAI API connectivity established
- **Blocker**: API parameter formatting causing 400 rejections

### **Priority**: HIGH - Critical for dynamic flow functionality

---
