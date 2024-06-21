document.getElementById('join-room').addEventListener('click', joinRoom);
document.getElementById('send-message').addEventListener('click', sendMessage);

let ws;
let room;
let password;

function joinRoom() {
    room = document.getElementById('room-input').value;
    password = document.getElementById('room-password').value;
    if (!room) {
        alert('Please enter a room name');
        return;
    }

    document.getElementById('room-selection').style.display = 'none';
    document.getElementById('chat-room').style.display = 'flex';

    ws = new WebSocket(`ws://127.0.0.1:3030/receive/${room}`);

    ws.onmessage = (event) => {
        const msg = JSON.parse(event.data);
        const chatWindow = document.getElementById('chat-window');
        const messageDiv = document.createElement('div');
        messageDiv.innerHTML = `<span class="username">${msg.username}:</span> ${msg.content}`;
        chatWindow.appendChild(messageDiv);
        chatWindow.scrollTop = chatWindow.scrollHeight;
    };
}

function sendMessage() {
    const username = document.getElementById('username').value;
    const content = document.getElementById('message-input').value;
    if (!username || !content) {
        alert('Please enter a username and message');
        return;
    }

    fetch(`http://127.0.0.1:3030/send/${room}`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify({ username, content, room, password }),
    });

    document.getElementById('message-input').value = '';
}
