// Grid defaults.
var CHAR_ALIVE = '■';
var CHAR_DEAD = '□';

window.onload = function() {
    var isOddMsg = false;
    var addMessage = function(msg) {
        messages.innerHTML +=
            (isOddMsg ? '<li class="message odd">' : '<li class="message">') + msg + '</li>';
        isOddMsg = !isOddMsg;
    };

    var gameArea = document.getElementById('game-area');

    var gridForm = document.getElementById('grid-form');
    var gridField = document.getElementById('grid-field');

    var gridOutput = document.getElementById('grid-output');
    var messages = document.getElementById('messages');

    var socket = new WebSocket('ws://localhost:3012');
    socket.onopen = function() {
        addMessage('Connected to game server.');
        socket.send('ping');
    };
    socket.onclose = function() {
        addMessage('Disconnected from game server.');
    };
    socket.onerror = function(error) {
        console.log('WebSocket Error: ' + error);
    };

    socket.onmessage = function(event) {
        var data = JSON.parse(event.data);
        if (data.status !== null)
            addMessage(data.status);
        if (data.pattern !== null)
            gridOutput.innerHTML = data.pattern
                .replace(/(\.)/g, CHAR_DEAD)
                .replace(/(x)/g, CHAR_ALIVE);
        setTimeout(function() {
            socket.send('ping');
        }, 500);
    };

    gridForm.onsubmit = function(e) {
        e.preventDefault();
        gameArea.scrollIntoView({
            block: 'start',
            inline: 'nearest',
            behavior: 'smooth'
        });
        var cmd = 'new-grid ' + gridField.value;
        socket.send(cmd);
        return false;
    };

    document.addEventListener('click', function(event) {
        if (event.target.classList.contains('command-btn')) {
            socket.send(event.target.value);
        }
    });
};
