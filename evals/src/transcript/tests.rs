use super::*;

const RUN: &str = concat!(
    "not json\n",
    "\"a string\"\n",
    "{\"type\": \"system\", \"subtype\": \"permission_denied\", \"message\": \"denied\"}\n",
    "{\"type\": \"assistant\", \"message\": {\"content\": [{\"type\": \"text\", \"text\": \"Looking.\"}, {\"type\": \"tool_use\", \"name\": \"Skill\", \"input\": {\"skill\": \"ground\"}}]}}\n",
    "{\"type\": \"user\", \"message\": {\"content\": [{\"type\": \"tool_result\", \"content\": \"found it\"}]}}\n",
    "{\"type\": \"user\", \"message\": {\"content\": [{\"type\": \"tool_result\", \"content\": [{\"type\": \"text\", \"text\": \"listed\"}]}]}}\n",
    "{\"type\": \"user\", \"message\": {\"content\": \"plain prompt\"}}\n",
    "{\"type\": \"assistant\", \"parent_tool_use_id\": \"t1\", \"message\": {\"content\": [{\"type\": \"text\", \"text\": \"subagent\"}]}}\n",
    "{\"type\": \"assistant\", \"message\": {\"content\": [{\"type\": \"thinking\", \"text\": \"hm\"}, \"stray\", {\"type\": \"text\", \"text\": \"Done.\"}]}}\n",
    "{\"type\": \"result\", \"session_id\": \"s-1\"}\n",
    "{\"type\": \"result\", \"session_id\": \"s-2\"}\n",
);

#[test]
fn events_keep_only_json_objects() {
    assert_eq!(events(RUN).len(), 9);
}

#[test]
fn digest_has_one_line_per_text_tool_call_and_result() {
    assert_eq!(
        digest(RUN),
        [
            "ASSISTANT: Looking.",
            "TOOL Skill: {\"skill\":\"ground\"}",
            "RESULT: found it",
            "RESULT: [{\"text\":\"listed\",\"type\":\"text\"}]",
            "ASSISTANT: subagent",
            "ASSISTANT: Done.",
        ]
    );
}

#[test]
fn digest_clips_long_tool_input_and_results() {
    let long = "x".repeat(600);
    let run = format!(
        "{{\"type\": \"assistant\", \"message\": {{\"content\": [{{\"type\": \"tool_use\", \"name\": \"Bash\", \"input\": {{\"command\": \"{long}\"}}}}]}}}}\n\
         {{\"type\": \"user\", \"message\": {{\"content\": [{{\"type\": \"tool_result\", \"content\": \"{long}\"}}]}}}}\n"
    );
    let lines = digest(&run);
    assert_eq!(lines[0].chars().count(), "TOOL Bash: ".len() + 500);
    assert_eq!(lines[1].chars().count(), "RESULT: ".len() + 500);
}

#[test]
fn said_is_the_main_agents_text_only() {
    assert_eq!(said(RUN), ["Looking.", "Done."]);
}

#[test]
fn session_id_is_from_the_last_result() {
    assert_eq!(session_id(RUN).as_deref(), Some("s-2"));
    assert_eq!(session_id("{\"type\": \"assistant\"}\n"), None);
}

#[test]
fn invoked_finds_assistant_skill_calls_only() {
    assert!(invoked(RUN, "ground"));
    assert!(!invoked(RUN, "prove"));
    let echoed = "{\"type\": \"user\", \"message\": {\"content\": [{\"name\": \"Skill\", \"input\": {\"skill\": \"prove\"}}]}}\n";
    assert!(!invoked(echoed, "prove"));
    let other_tool = "{\"type\": \"assistant\", \"message\": {\"content\": [{\"name\": \"Task\", \"input\": {\"skill\": \"prove\"}}]}}\n";
    assert!(!invoked(other_tool, "prove"));
}
