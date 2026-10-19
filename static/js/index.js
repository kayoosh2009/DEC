// ─── State ────────────────────────────────────────────────────────────────────

let menuOpen = false;
let popupOpen = false;
let currentChatId = null;
let currentUser = null;

// ─── Init ─────────────────────────────────────────────────────────────────────

function initChat(user) {
    currentUser = user;

    // Session ID в navbar
    let id = sessionStorage.getItem('dec_id');
    if (!id) {
        id = 'DEC-' + Math.floor(1000 + Math.random() * 9000);
        sessionStorage.setItem('dec_id', id);
    }
    document.getElementById('navbarId').textContent = id;

    // Закрывать popup при клике вне
    document.addEventListener('click', e => {
        const popup = document.getElementById('logoutPopup');
        const avatar = document.getElementById('userAvatar');
        if (popupOpen && !popup.contains(e.target) && e.target !== avatar) {
            popupOpen = false;
            popup.classList.remove('visible');
        }
    });

    const name = user.displayName ? user.displayName.split(' ')[0] : null;
    if (name) {
        document.getElementById('welcomeText').textContent = `What are we doing today, ${name}?`;
    }

    newChat();
    loadChatList();
}

// ─── Menu ─────────────────────────────────────────────────────────────────────

function toggleMenu() {
    menuOpen = !menuOpen;
    document.getElementById('menuIcon').classList.toggle('open', menuOpen);
    document.getElementById('sideMenu').classList.toggle('open', menuOpen);
}

function toggleLogoutPopup() {
    popupOpen = !popupOpen;
    document.getElementById('logoutPopup').classList.toggle('visible', popupOpen);
}

// ─── Chat ─────────────────────────────────────────────────────────────────────

let currentChatTitle = null;
let isFirstMessage = true;

function newChat() {
    currentChatId = crypto.randomUUID();
    currentChatTitle = null;
    isFirstMessage = true;
    document.getElementById('chatMessages').innerHTML = '';
    document.getElementById('chatWelcome').classList.remove('hidden');
    document.getElementById('chatInput').value = '';
}

async function loadChatList() {
    const idToken = await currentUser.getIdToken();
    const resp = await fetch(`/chat/list?uid=${currentUser.uid}&id_token=${encodeURIComponent(idToken)}`);
    if (!resp.ok) return;

    const chats = await resp.json();

    // Закреплённые вверху, остальные по дате
    chats.sort((a, b) => {
        if (a.pinned && !b.pinned) return -1;
        if (!a.pinned && b.pinned) return 1;
        return b.timestamp.localeCompare(a.timestamp);
    });

    const list = document.getElementById('chatList');
    list.innerHTML = '';

    for (const chat of chats) {
        const item = document.createElement('div');
        item.classList.add('chat-item');
        if (chat.pinned) item.classList.add('chat-item-pinned');
        item.dataset.chatId = chat.chat_id;

        const row = document.createElement('div');
        row.classList.add('chat-item-row');

        const body = document.createElement('div');
        body.classList.add('chat-item-body');
        body.onclick = () => openChat(chat.chat_id);

        const title = document.createElement('div');
        title.classList.add('chat-item-title');
        title.textContent = chat.title;

        const date = document.createElement('div');
        date.classList.add('chat-item-date');
        date.textContent = new Date(chat.timestamp).toLocaleDateString('ru-RU', {
            day: 'numeric', month: 'short', hour: '2-digit', minute: '2-digit'
        });

        body.appendChild(title);
        body.appendChild(date);

        const dotsBtn = document.createElement('button');
        dotsBtn.classList.add('chat-dots-btn');
        dotsBtn.innerHTML = '<img src="img/dots.png" alt="...">';
        dotsBtn.onclick = (e) => {
            e.stopPropagation();
            showChatMenu(e, chat);
        };

        row.appendChild(body);
        row.appendChild(dotsBtn);
        item.appendChild(row);
        list.appendChild(item);
    }
}

// ─── Context menu ─────────────────────────────────────────────────────────────

let contextMenuChat = null;

function showChatMenu(e, chat) {
    closeContextMenu();

    contextMenuChat = chat;
    const menu = document.getElementById('chatContextMenu');

    // Пункт закрепить/открепить
    document.getElementById('menuPinLabel').textContent = chat.pinned ? 'Unpin' : 'Pin';

    menu.classList.add('visible');

    // Позиционируем около кнопки
    const rect = e.target.closest('button').getBoundingClientRect();
    menu.style.top = rect.bottom + 4 + 'px';
    menu.style.left = rect.left - 120 + 'px';
}

function closeContextMenu() {
    document.getElementById('chatContextMenu').classList.remove('visible');
    contextMenuChat = null;
}

async function pinChat() {
    if (!contextMenuChat) return;
    const idToken = await currentUser.getIdToken();
    const newPinned = !contextMenuChat.pinned;

    await fetch('/chat/save', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            chat_id: contextMenuChat.chat_id,
            title: contextMenuChat.title,
            timestamp: contextMenuChat.timestamp,
            pinned: newPinned,
            id_token: idToken,
            uid: currentUser.uid,
        })
    });

    closeContextMenu();
    await loadChatList();
}

function startRename() {
    if (!contextMenuChat) return;
    closeContextMenu();

    const item = document.querySelector(`[data-chat-id="${contextMenuChat.chat_id}"]`);
    if (!item) return;

    const titleEl = item.querySelector('.chat-item-title');
    const currentTitle = titleEl.textContent.replace('📌 ', '');

    const input = document.createElement('input');
    input.classList.add('chat-rename-input');
    input.value = currentTitle;

    titleEl.replaceWith(input);
    input.focus();
    input.select();

    input.onblur = () => finishRename(input, contextMenuChat?.chat_id || item.dataset.chatId);
    input.onkeydown = (e) => {
        if (e.key === 'Enter') input.blur();
        if (e.key === 'Escape') loadChatList();
    };
}

async function finishRename(input, chatId) {
    const newTitle = input.value.trim();
    if (!newTitle) { await loadChatList(); return; }

    const idToken = await currentUser.getIdToken();
    await fetch('/chat/save', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            chat_id: chatId,
            title: newTitle,
            timestamp: new Date().toISOString(),
            id_token: idToken,
            uid: currentUser.uid,
        })
    });

    await loadChatList();
}

async function deleteChat() {
    if (!contextMenuChat) return;
    const chat = contextMenuChat;
    closeContextMenu();

    const idToken = await currentUser.getIdToken();

    await fetch('/chat/delete', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            chat_id: chat.chat_id,
            id_token: idToken,
            uid: currentUser.uid,
        })
    });

    if (currentChatId === chat.chat_id) newChat();
    await loadChatList();
}

// Закрывать меню при клике вне
document.addEventListener('click', () => closeContextMenu());

async function openChat(chatId) {
    currentChatId = chatId;
    isFirstMessage = false;
    document.getElementById('chatMessages').innerHTML = '';
    document.getElementById('chatWelcome').classList.add('hidden');

    const idToken = await currentUser.getIdToken();
    const resp = await fetch(`/chat/messages?uid=${currentUser.uid}&id_token=${encodeURIComponent(idToken)}&chat_id=${chatId}`);
    if (!resp.ok) return;

    const msgs = await resp.json();
    for (const msg of msgs) {
        appendMessage(msg.role === 'assistant' ? 'ai' : msg.role, msg.content, msg.timestamp);
    }
}

function autoResize(el) {
    el.style.height = 'auto';
    el.style.height = Math.min(el.scrollHeight, 160) + 'px';
}

function handleKey(e) {
    if (e.key === 'Enter' && !e.shiftKey) {
        e.preventDefault();
        sendMessage();
    }
}

function appendMessage(role, text, timestamp = null) {
    const container = document.getElementById('chatMessages');
    const msgId = crypto.randomUUID();
    const msgTimestamp = timestamp || new Date().toISOString();

    const wrap = document.createElement('div');
    wrap.classList.add('message', role === 'user' ? 'message-user' : 'message-ai');
    wrap.dataset.msgId = msgId;
    wrap.dataset.timestamp = msgTimestamp;
    wrap.dataset.role = role;
    wrap.dataset.content = text;

    const bubble = document.createElement('div');
    bubble.classList.add('message-bubble');
    bubble.textContent = text;

    const actions = document.createElement('div');
    actions.classList.add('message-actions');

    const starBtn = document.createElement('button');
    starBtn.classList.add('star-btn');
    starBtn.textContent = '☆';
    starBtn.onclick = () => toggleStar(starBtn, msgId, role, text, msgTimestamp);

    actions.appendChild(starBtn);
    wrap.appendChild(bubble);
    wrap.appendChild(actions);
    container.appendChild(wrap);
    container.scrollTop = container.scrollHeight;
}

async function toggleStar(btn, msgId, role, content, timestamp) {
    const isStarred = btn.classList.toggle('starred');
    btn.textContent = isStarred ? '★' : '☆';

    if (!isStarred) return;

    const idToken = await currentUser.getIdToken();

    await fetch('/chat/star', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            chat_id: currentChatId,
            msg_id: msgId,
            message: { role, content },
            timestamp,
            id_token: idToken,
            uid: currentUser.uid,
        })
    });
}

async function sendMessage() {
    const input = document.getElementById('chatInput');
    const text = input.value.trim();
    if (!text || !currentUser) return;

    document.getElementById('chatWelcome').classList.add('hidden');

    input.value = '';
    input.style.height = 'auto';

    appendMessage('user', text);

    const idToken = await currentUser.getIdToken();
    const timestamp = new Date().toISOString();

// Если первое сообщение — генерируем заголовок
    if (isFirstMessage) {
        isFirstMessage = false;
        const titleResp = await fetch('/chat/send', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                chat_id: crypto.randomUUID(),
                message: `Give this chat a short title (max 5 words) based on this first message. Reply with the title only, no punctuation: "${text}"`,
                timestamp,
                id_token: idToken,
                uid: currentUser.uid,
            })
        });
        if (titleResp.ok) {
            const titleData = await titleResp.json();
            currentChatTitle = titleData.message.trim();
            await fetch('/chat/save', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    chat_id: currentChatId,
                    title: currentChatTitle,
                    timestamp,
                    id_token: idToken,
                    uid: currentUser.uid,
                })
            });
            await loadChatList();
        }
    }

    // Показываем индикатор печати
    showTypingIndicator();

    const resp = await fetch('/chat/send', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            chat_id: currentChatId,
            message: text,
            timestamp,
            id_token: idToken,
            uid: currentUser.uid,
        })
    });

    removeTypingIndicator();

    if (!resp.ok) {
        appendMessage('ai', 'Error: ' + resp.status);
        return;
    }

    const data = await resp.json();
    appendMessage('ai', data.message, data.timestamp);
    await loadChatList();
}

function showTypingIndicator() {
    const container = document.getElementById('chatMessages');
    const wrap = document.createElement('div');
    wrap.classList.add('message', 'message-ai');
    wrap.id = 'typingIndicator';

    const bubble = document.createElement('div');
    bubble.classList.add('message-bubble', 'typing-bubble');
    bubble.innerHTML = '<span class="typing-dot"></span><span class="typing-dot"></span><span class="typing-dot"></span>';

    wrap.appendChild(bubble);
    container.appendChild(wrap);
    container.scrollTop = container.scrollHeight;
}

function removeTypingIndicator() {
    const el = document.getElementById('typingIndicator');
    if (el) el.remove();
}