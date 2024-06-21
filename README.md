```
┌────────────────────────────────────────────────────────────────────────────┐
│                                Good to know                                │
├────────────────────────────────────────────────────────────────────────────┤
│Channels can have _one_ receiver of messages, but many can send to that     │
│receiver. In our application, we have multiple services sending events,     │
│sometimes over the same channel, but only ONE part of the program can       │
│receive and act on these events.                                            │
│This means we sometimes need to use the fan-out pattern, where one part of  │
│our service receives an event, and then forwards events to other parts of   │
│the system (we use the EventService for this)                               │
│                                                                            │
└────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│                                  Main loop                                  │
├─────────────────────────────────────────────────────────────────────────────┤
│Responsible for initializing channels and creating puzzle pieces.            │
│                                                                             │
│- Creates all the stuff needed in order to have a psuedo terminal (but does  │
│not render it!)                                                              │
│                                                                             │
│- Creates AppState, where we store general application state, such as which  │
│mode the user is in (chat or terminal), if the app is running and whatever   │
│the AI has responded with etc.                                               │
│                                                                             │
│- Creates a UiService, a service which is responsible for rendering UI. This │
│service also consumes the EventService.                                      │
│Any service, such as key presses, terminal updates, chat responses from the  │
│AI can trigger their own Event and send to the EventService. The EventService│
│is responsible for _when_ to render a new frame.                             │
│                                                                             │
│- Creates a ChatService, a service that runs in the background, and listen   │
│for new questions, and communicates with the AI libraries. This service sends│
│Events to the EventService(Which is the UI service), which triggers a        │
│re-render.                                                                   │
└──────────────────┬────────────────────────────────────────┬─────────────────┘
                   │                   │                    │
                   │                   │                    │
                   ▼                   │                    ▼
┌─────────────────────────────────────┐│ ┌────────────────────────────────────┐
│             ChatService             ││ │              Terminal              │
├─────────────────────────────────────┤│ ├────────────────────────────────────┤
│Listen to Action-events              ││ │We do most of the grun-work in the  │
│(action_receiver)                    ││ │main function, by calling the       │
│Sends Events to the UI for re-render ││ │terminal_util::new function. This   │
│                                     ││ │basically sets up everything related│
│Base functionality:                  ││ │to the terminal, except rendering it│
│A background task which we can       ││ │(this is done by the UI)            │
│trigger chats with, by sending       ││ │We can send events to the terminal  │
│Action-events (from anywhere we      ││ │by using the `terminal_sender`      │
│want!). So far the Action-event is   ││ │queue. This queue expects bytes, so │
│just a string, but we might want to  ││ │in order to send a character `a` to │
│store state etc as well. We'll see :)││ │the terminal, we must say something │
└─────────────────────────────────────┘│ │like                                │
                                       │ │`terminal_sender.send(Bytes::from('a│
                                       │ │'.to_string().into_bytes())`.       │
                                       │ │Since crossterm is our backend, and │
                                       │ │any events sent from the CLI (such  │
                                       │ │as key strokes, mouse movement etc) │
                                       │ │is handled by the EventService, the │
                                       │ └────────────────────────────────────┘
                                       ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                UiService (which also owns the EventService)                 │
├─────────────────────────────────────────────────────────────────────────────┤
│UiService:                                                                   │
│                                                                             │
│The UiService is the heart of the program, and responsible for rendering the │
│UI, but also handling any user-events, and propagate these events to other   │
│parts of the service (such as, if the user writes 'a' while being in the     │
│Mode::Terminal, the 'a' should be sent to the TerminalService. If the user is│
│in Mode::Chat, this 'a' should be sent to the chat input box)                │
│                                                                             │
│There's a few ways a new frame can be rendered, since the UI also owns the   │
│EventService, which listen to events from different parts of the system. At  │
│the time of writing this, it listens to the cross_term-events(user           │
│input/movement) as mentioned above, but also any Events sent through the     │
│event_channel. Multiple parts of the program can send events, such as the    │
│ChatService will send an event when a new part of the ChatGPT message has    │
│been received.                                                               │
│                                                                             │
│In order to update the UI one of the following things must happen:           │
│- Event received from another part of the app                                │
│- Crossterm(user input) event is received                                    │
│- No event is received from other parts of the system, and a 10ms timeout is │
│reached                                                                      │
│                                                                             │
│The `EventService::next`-function handles this. The UI                       │
│`service::start`-function will go through any event, and render a frame, and │
│potentially also forward events to other parts.                              │
│                                                                             │
│An example is when a user sends 'a', this is a crossterm_event sent to the   │
│EventService, which in turn sends an Event, which is caught by by the        │
│UiService::start function. Here, based on what the app state is, we can      │
│either send this to the terminal, or to the chat.                            │
└─────────────────────────────────────────────────────────────────────────────┘
```
