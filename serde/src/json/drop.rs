use super::Value;

pub fn safe_drop(value: Value) {
    match value {
        Value::Array(_) | Value::Object(_) => {}
        _ => return,
    }

    let mut stack = vec![value];

    while let Some(value) = stack.pop() {
        match value {
            Value::Array(vec) => {
                for child in vec {
                    stack.push(child);
                }
            }
            Value::Object(map) => {
                for (_, child) in map {
                    stack.push(child);
                }
            }
            _ => {}
        }
    }
}
