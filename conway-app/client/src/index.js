'use strict';

/*
 * Constants.
 */
const DEFAULT_PATTERN = `
............x............
............x............
............x............
xxx......................
..x......................
.x.......................
.........................
.....xxx..xxxxx..xxx.....
.........................
.......................x.
......................x..
......................xxx
............x............
............x............
............x............
`;

const CHAR_ALIVE = '■';
const CHAR_DEAD = '□';

const MSG_CONNECTED = 'Connected';
const MSG_STATUS = 'Status';
const MSG_GRID = 'Grid';
const MSG_ERROR = 'Error';

window.onload = function() {
  /*
   * Get elements.
   */
  const $gameArea = document.getElementById('game-area');
  const $gridForm = document.getElementById('grid-form');
  const $gridField = document.getElementById('grid-field');
  const $reconnectBtn = document.getElementById('reconnect-btn');
  const $grid = document.getElementById('grid-area');
  const $messages = document.getElementById('messages');

  /*
   * Set default pattern
   */
  $gridField.innerHTML = DEFAULT_PATTERN.trim();

  /*
   * Define utility to add message to message box.
   */
  let isOddMsg = false;
  const addMessage = function(msg) {
    const isScrolledDown = $messages.scrollHeight - $messages.clientHeight <= $messages.scrollTop;

    const elem = document.createElement('li');
    elem.setAttribute('class', isOddMsg ? 'message odd' : 'message');
    elem.textContent = msg;
    $messages.appendChild(elem);

    isOddMsg = !isOddMsg;

      // If the message box was already scrolled down, auto-scroll down to reveal new message.
    if (isScrolledDown)
      $messages.scrollTop = $messages.scrollHeight - $messages.clientHeight;
  };

  /*
   * Setup WebSocket.
   */
  const socket = new WebSocket('ws://localhost:3012');
  socket.onclose = function() {
    addMessage('Disconnected from game server.');
  };
  socket.onerror = function(error) {
    console.log('Error communicating with game server: ' + error);
  };
  socket.onmessage = function(event) {
    const msg = JSON.parse(event.data);
    switch (msg.kind) {
    case MSG_CONNECTED:
      addMessage('Connected to game server.');
      break;
    case MSG_STATUS:
      addMessage(msg.content);
      break;
    case MSG_GRID:
      $grid.innerHTML = msg.content.trim()
                .replace(/(\.)/g, CHAR_DEAD)
                .replace(/(x)/g, CHAR_ALIVE);
      break;
    case MSG_ERROR:
      addMessage('ERROR: ' + msg.content);
      break;
    }
    setTimeout(function() {
      socket.send('ping');
    }, 500);
  };

  /*
   * Setup grid form.
   */
  $gridForm.onsubmit = function(event) {
    event.preventDefault();
    $gameArea.scrollIntoView({
      block: 'start',
      inline: 'nearest',
    });

      // Build the Settings object.
      // Use hardcoded values for `char_alive` and `char_dead`.
    const settings = { char_alive: CHAR_ALIVE, char_dead: CHAR_DEAD };

      // Compute width and height to fit containing element.
    const fontSize = parseFloat(getComputedStyle($grid).getPropertyValue('font-size'));
    settings.width = Math.ceil($grid.clientWidth / (fontSize * 0.61));
    settings.height = Math.ceil($grid.clientHeight / (fontSize * 0.51));

    const fields = event.target.elements;

      // Fetch `delay` from form and turn it into Duration json repr. for the backend.
    const delay_ms = fields['tick-delay'].value;
    const delay_secs = Math.trunc(delay_ms / 1000);
    const delay_nanos = (delay_ms - (delay_secs * 1000)) * 1000000;
    settings.delay = { secs: delay_secs, nanos: delay_nanos };

      // Fetch `view` from form.
    settings.view = fields['view'].value;

      // Send message.
    const payload = JSON.stringify({ pattern: $gridField.value, settings: settings });
    const msg = 'new-grid ' + payload;

    if (socket.readyState !== socket.OPEN) {
      addMessage('Disconnected from game server.');
      reconnect();
    }
    socket.send(msg);
  };

  const reconnect = function(silent) {
    if (!silent)
      addMessage('Attempting to reconnect...');
      // FIXME -> once WebSocket loses connection a new one must be created
      // socket.io will fix this
    socket.dispatchEvent(new Event('open'));
  };

  /*
   * Reconnect button
   */
  $reconnectBtn.onclick = function() {
    if (socket.readyState === socket.OPEN)
      addMessage('Already connected to game server.');
    reconnect(true);
  };

  /*
   * Setup control panel.
   */
  document.querySelectorAll('#control-panel button').forEach(function(button) {
    button.onclick = function(event) {
      if (socket.readyState !== socket.OPEN) {
        addMessage('Disconnected from game server.');
      }
      socket.send(event.target.value);
    };
  });
};
