window.onload = function() {
    var commandForm = document.getElementById('command-form');
    var commandField = document.getElementById('command-field');
    var gridForm = document.getElementById('grid-form');
    var gridField = document.getElementById('grid-field');

    var btnScrollLeft = document.getElementById('btn-scroll-left');
    var btnScrollRight = document.getElementById('btn-scroll-right');
    var btnScrollUp = document.getElementById('btn-scroll-up');
    var btnScrollDown = document.getElementById('btn-scroll-down');

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
        if (data.status !== null)
            statusOutput.innerHTML = data.status;
        if (data.pattern !== null)
            gridOutput.innerHTML = data.pattern;
        setTimeout(function() {
            socket.send('ping');
        }, 500);
    };

    gridForm.onsubmit = function(e) {
        e.preventDefault();
        var cmd = 'restart ' + gridField.value;
        socket.send(cmd);
        return false;
    };

    commandForm.onsubmit = function(e) {
        e.preventDefault();
        var cmd = commandField.value;
        socket.send(cmd);
        return false;
    };

    btnScrollLeft.onclick = function() {
        socket.send(btnScrollLeft.value);
    };
    btnScrollRight.onclick = function() {
        socket.send(btnScrollRight.value);
    };
    btnScrollUp.onclick = function() {
        socket.send(btnScrollUp.value);
    };
    btnScrollDown.onclick = function() {
        socket.send(btnScrollDown.value);
    };
};
