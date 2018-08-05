window.onload = function() {
    var commandForm = document.getElementById('command-form');
    var commandField = document.getElementById('command');
    var gridInput = document.getElementById('starting-grid');
    var restartButton = document.getElementById('submit-pattern');
    var outputField = document.getElementById('game-output');
    var socketStatus = document.getElementById('status');

    var socket = new WebSocket('ws://localhost:3012');
    socket.onopen = function(event) {
        socketStatus.innerHTML = 'Connected to: ' + event.currentTarget.url;
        socketStatus.className = 'open';
        socket.send('ping');
    };
    socket.onclose = function() {
        socketStatus.innerHTML = 'Disconnected from WebSocket.';
        socketStatus.className = 'closed';
    };
    socket.onerror = function(error) {
        console.log('WebSocket Error: ' + error);
    };
    socket.onmessage = function(event) {
        var data = JSON.parse(event.data);
        if (data.status !== null)
            socketStatus.innerHTML = data.status;
        if (data.pattern !== null)
            outputField.innerHTML = data.pattern;
        setTimeout(function() {
            socket.send('ping');
        }, 500);
    };

    restartButton.onclick = function() {
        var cmd = restartButton.value + ' ' + gridInput.value;
        socket.send(cmd);
        return false;
    };

    commandForm.onsubmit = function(e) {
        e.preventDefault();
        var cmd = commandField.value;
        socket.send(cmd);
        return false;
    };
};
