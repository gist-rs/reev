import json
import sys

# Read enhanced OTEL JSONL file
file_path = sys.argv[1] if len(sys.argv) > 1 else "logs/sessions/enhanced_otel_306114a3-3d36-43bb-ac40-335fef6307ac.jsonl"

tool_calls = []
with open(file_path, 'r') as f:
    for line in f:
        data = json.loads(line)
        
        # Extract tool calls from OTEL data
        if data.get('event_type') == 'ToolInput':
            tool_input = data.get('tool_input', {})
            if tool_input:
                tool_calls.append({
                    'tool_name': tool_input.get('tool_name'),
                    'start_time': data.get('timestamp'),
                    'input': tool_input.get('tool_args', {}),
                    'duration_ms': data.get('timing', {}).get('step_timeuse_ms', 0)
                })
        elif data.get('event_type') == 'ToolOutput':
            tool_output = data.get('tool_output', {})
            if tool_calls:  # Add output to last tool call
                tool_calls[-1]['output'] = tool_output.get('results', '')
                tool_calls[-1]['success'] = tool_output.get('success', False)

print(f"Found {len(tool_calls)} tool calls:")
for i, call in enumerate(tool_calls):
    print(f"  {i+1}. {call.get('tool_name', 'Unknown')} - {call.get('input', {})}")

# Create simple YML content
yml_content = f"""session_id: 306114a3-3d36-43bb-ac40-335fef6307ac
tool_calls:
"""

for call in tool_calls:
    yml_content += f"""  - tool_name: {call.get('tool_name', 'Unknown')}
    start_time: "{call.get('start_time', '')}"
    input: {call.get('input', {})}
    duration_ms: {call.get('duration_ms', 0)}
"""

# Write YML file
yml_path = file_path.replace('.jsonl', '.yml')
with open(yml_path, 'w') as f:
    f.write(yml_content)

print(f"\nConverted to: {yml_path}")
