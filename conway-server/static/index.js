// Grid defaults.
var CHAR_ALIVE = '■';
var CHAR_DEAD = '□';

window.onload = function() {
    var gameArea = document.getElementById('game-area');

    var gridForm = document.getElementById('grid-form');
    var gridField = document.getElementById('grid-field');

    var gridOutput = document.getElementById('grid-output');
    var statusOutput = document.getElementById('status-output');

    var socket = new WebSocket('ws://localhost:3012');
    socket.onopen = function(event) {
        statusOutput.innerHTML = 'Connected to: ' + event.currentTarget.url;
        statusOutput.className = 'open';
        socket.send('ping');
    };
    socket.onclose = function() {
        statusOutput.innerHTML = 'Disconnected from WebSocket.';
        statusOutput.className = 'closed';
    };
    socket.onerror = function(error) {
        console.log('WebSocket Error: ' + error);
    };
    socket.onmessage = function(event) {
        var data = JSON.parse(event.data);
        statusOutput.innerHTML = data.status;
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
