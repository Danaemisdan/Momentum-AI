import re

with open('src/main.rs', 'r', encoding='utf-8') as f:
    content = f.read()

# Add active_chat_task variable
content = content.replace(
    'let active_task: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>> = Arc::new(Mutex::new(None));',
    'let active_task: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>> = Arc::new(Mutex::new(None));\n    let active_chat_task: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>> = Arc::new(Mutex::new(None));'
)

# Replace the chat tokio::spawn with a tracked handle
old_spawn = r'''tokio::spawn\(async move \{
                            let mut msg_with_vision = message.clone\(\);'''
new_spawn = '''let chat_handle = tokio::spawn(async move {
                            let mut msg_with_vision = message.clone();'''
content = re.sub(old_spawn, new_spawn, content)

# Inject the cancellation logic before the spawn, and assigning the handle
old_chat_block = r'''let msg_lower = message.to_lowercase\(\);
                        let b_mu_chat = browser_mu.clone\(\);
                        
                        let chat_handle = tokio::spawn\(async move \{'''
new_chat_block = '''let msg_lower = message.to_lowercase();
                        let b_mu_chat = browser_mu.clone();
                        
                        let mut guard = active_chat_task.lock().await;
                        if let Some(old_handle) = guard.take() {
                            old_handle.abort();
                        }
                        
                        let chat_handle = tokio::spawn(async move {'''
content = re.sub(old_chat_block, new_chat_block, content)

# Store the handle
old_handle_storage = r'''// Do not filter out short messages like "hi"'''
new_handle_storage = '''*guard = Some(chat_handle);
                        
                        // Do not filter out short messages like "hi"'''
content = content.replace(old_handle_storage, new_handle_storage)


with open('src/main.rs', 'w', encoding='utf-8') as f:
    f.write(content)
