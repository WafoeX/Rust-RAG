// Tab switching
document.querySelectorAll('.tab').forEach(tab => {
    tab.addEventListener('click', () => {
        document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
        tab.classList.add('active');
        document.querySelectorAll('.panel').forEach(p => p.classList.remove('active'));
        document.getElementById('panel-' + tab.dataset.tab).classList.add('active');
    });
});

// ---- Upload ----
const dropZone = document.getElementById('drop-zone');
const fileInput = document.getElementById('file-input');
const uploadStatus = document.getElementById('upload-status');
const uploadMsg = document.getElementById('upload-msg');
const uploadResult = document.getElementById('upload-result');
const uploadError = document.getElementById('upload-error');
const uploadErrorMsg = document.getElementById('upload-error-msg');

// Click to select file
dropZone.addEventListener('click', () => {
    resetUploadView();
    fileInput.click();
});

fileInput.addEventListener('change', () => {
    if (fileInput.files.length > 0) {
        uploadFile(fileInput.files[0]);
    }
});

// Drag and drop
dropZone.addEventListener('dragover', e => {
    e.preventDefault();
    dropZone.classList.add('drag-over');
});

dropZone.addEventListener('dragleave', () => {
    dropZone.classList.remove('drag-over');
});

dropZone.addEventListener('drop', e => {
    e.preventDefault();
    dropZone.classList.remove('drag-over');
    const file = e.dataTransfer.files[0];
    if (file) uploadFile(file);
});

// Re-upload buttons
document.getElementById('reset-upload-btn').addEventListener('click', resetUploadView);
document.getElementById('retry-upload-btn').addEventListener('click', resetUploadView);

function resetUploadView() {
    fileInput.value = '';
    dropZone.style.display = 'block';
    uploadStatus.style.display = 'none';
    uploadResult.style.display = 'none';
    uploadError.style.display = 'none';
}

function showUploadView(view) {
    dropZone.style.display = view === 'drop' ? 'block' : 'none';
    uploadStatus.style.display = view === 'status' ? 'block' : 'none';
    uploadResult.style.display = view === 'result' ? 'block' : 'none';
    uploadError.style.display = view === 'error' ? 'block' : 'none';
}

function showUploadError(msg) {
    uploadErrorMsg.textContent = '错误：' + msg;
    showUploadView('error');
}

async function uploadFile(file) {
    if (!file) return;

    const ext = file.name.split('.').pop().toLowerCase();
    if (!['txt', 'md', 'markdown', 'pdf'].includes(ext)) {
        showUploadError('不支持的文件格式。请上传 TXT、Markdown 或 PDF 文件。');
        return;
    }

    showUploadView('status');
    uploadMsg.textContent = '正在上传并处理文档：' + file.name;

    const formData = new FormData();
    formData.append('file', file);

    try {
        const resp = await fetch('/api/documents/upload', {
            method: 'POST',
            body: formData,
        });

        if (!resp.ok) {
            const err = await resp.json();
            throw new Error(err.error || '上传失败');
        }

        const data = await resp.json();
        document.getElementById('result-id').textContent = data.document_id;
        document.getElementById('result-name').textContent = data.file_name;
        document.getElementById('result-chunks').textContent = data.chunk_count;
        showUploadView('result');
        fileInput.value = '';
    } catch (e) {
        showUploadError(e.message);
        fileInput.value = '';
    }
}

// ---- Chat ----
const chatMessages = document.getElementById('chat-messages');
const questionInput = document.getElementById('question-input');
const sendBtn = document.getElementById('send-btn');
const topKInput = document.getElementById('top-k');

// Conversation history: only stores Q&A text, not retrieved chunks.
// Capped at 10 rounds (20 messages) to keep prompt size manageable.
const MAX_HISTORY_ROUNDS = 10;
let conversationHistory = [];

sendBtn.addEventListener('click', sendQuestion);
questionInput.addEventListener('keydown', e => {
    if (e.key === 'Enter' && !e.shiftKey) {
        e.preventDefault();
        sendQuestion();
    }
});

async function sendQuestion() {
    const question = questionInput.value.trim();
    if (!question) return;

    const topK = parseInt(topKInput.value) || 5;

    addMessage('user', question);
    questionInput.value = '';

    const loadingMsg = addMessage('assistant loading', '');
    loadingMsg.innerHTML = '<div class="typing-indicator"><span></span><span></span><span></span></div>';
    scrollToBottom();

    sendBtn.disabled = true;
    questionInput.disabled = true;

    try {
        const body = {
            question,
            top_k: topK,
        };

        // Include conversation history if we have previous rounds
        if (conversationHistory.length > 0) {
            body.history = conversationHistory;
        }

        const resp = await fetch('/api/query', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(body),
        });

        if (!resp.ok) {
            const err = await resp.json();
            throw new Error(err.error || '查询失败');
        }

        const data = await resp.json();
        loadingMsg.remove();

        // Save this round to conversation history
        conversationHistory.push({ role: 'user', content: question });
        conversationHistory.push({ role: 'assistant', content: data.answer });

        // Cap history size
        const maxMessages = MAX_HISTORY_ROUNDS * 2;
        if (conversationHistory.length > maxMessages) {
            conversationHistory = conversationHistory.slice(-maxMessages);
        }

        let content = '<div class="answer-text">' + escapeHtml(data.answer) + '</div>';

        if (data.sources && data.sources.length > 0) {
            content += '<div class="sources"><details><summary>参考来源（' + data.sources.length + ' 条）</summary>';
            data.sources.forEach((s, i) => {
                content +=
                    '<div class="source-item">' +
                    '<div class="source-meta">[' + (i + 1) + '] ' + escapeHtml(s.file_name) +
                    ' &middot; 片段 #' + s.chunk_index +
                    ' &middot; 相关度 ' + (s.score * 100).toFixed(1) + '%</div>' +
                    '<div class="source-content">' + escapeHtml(s.content) + '</div>' +
                    '</div>';
            });
            content += '</details></div>';
        } else {
            content += '<div class="sources"><em>未找到相关片段。</em></div>';
        }

        addMessage('assistant', content);
    } catch (e) {
        loadingMsg.remove();
        addMessage('error', '查询失败：' + escapeHtml(e.message));
    }

    sendBtn.disabled = false;
    questionInput.disabled = false;
    questionInput.focus();
    scrollToBottom();
}

function addMessage(type, content) {
    const div = document.createElement('div');
    div.className = 'message ' + type;
    div.innerHTML = content;
    chatMessages.appendChild(div);
    return div;
}

function scrollToBottom() {
    chatMessages.scrollTop = chatMessages.scrollHeight;
}

function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}
