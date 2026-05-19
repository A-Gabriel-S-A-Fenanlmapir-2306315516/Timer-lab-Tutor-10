#[cfg(target_arch = "wasm32")]
mod app {
    use std::rc::Rc;

    use futures_channel::mpsc::{UnboundedSender, unbounded};
    use futures_util::{SinkExt, StreamExt, future, pin_mut};
    use gloo_net::websocket::{Message, futures::WebSocket};
    use wasm_bindgen_futures::spawn_local;
    use web_sys::{HtmlElement, HtmlInputElement, InputEvent, SubmitEvent};
    use yew::TargetCast;
    use yew::prelude::*;

    const WS_URL: &str = "ws://127.0.0.1:7878";
    const MAX_MESSAGES: usize = 80;

    #[derive(Clone, PartialEq)]
    struct ChatMessage {
        id: usize,
        author: String,
        body: String,
        variant: MessageVariant,
    }

    #[derive(Clone, Copy, PartialEq)]
    enum MessageVariant {
        Peer,
        Relay,
    }

    #[derive(Clone, PartialEq)]
    struct ChatState {
        messages: Vec<ChatMessage>,
        next_id: usize,
    }

    impl Default for ChatState {
        fn default() -> Self {
            Self {
                messages: Vec::new(),
                next_id: 1,
            }
        }
    }

    enum ChatAction {
        PushRaw(String),
        PushRelay(String),
        Clear,
    }

    impl Reducible for ChatState {
        type Action = ChatAction;

        fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
            let mut next = (*self).clone();

            match action {
                ChatAction::PushRaw(raw) => {
                    let (author, body, variant) = parse_incoming_message(&raw);
                    next.push(author, body, variant);
                }
                ChatAction::PushRelay(body) => {
                    next.push("relay".to_owned(), body, MessageVariant::Relay);
                }
                ChatAction::Clear => {
                    next.messages.clear();
                }
            }

            Rc::new(next)
        }
    }

    impl ChatState {
        fn push(&mut self, author: String, body: String, variant: MessageVariant) {
            self.messages.push(ChatMessage {
                id: self.next_id,
                author,
                body,
                variant,
            });
            self.next_id += 1;

            if self.messages.len() > MAX_MESSAGES {
                let overflow = self.messages.len() - MAX_MESSAGES;
                self.messages.drain(0..overflow);
            }
        }
    }

    #[derive(Clone, Copy, PartialEq)]
    enum ConnectionState {
        Connecting,
        Connected,
        Disconnected,
    }

    #[function_component(App)]
    fn app() -> Html {
        let chat = use_reducer(ChatState::default);
        let draft = use_state(String::new);
        let connection = use_state(|| ConnectionState::Connecting);
        let reconnects = use_state(|| 0_u32);
        let outbox = use_mut_ref(|| None::<UnboundedSender<String>>);
        let log_ref = use_node_ref();

        {
            let chat = chat.clone();
            let connection = connection.clone();
            let outbox = outbox.clone();
            let reconnect_key = *reconnects;

            use_effect_with(reconnect_key, move |_| {
                spawn_local(async move {
                    connection.set(ConnectionState::Connecting);
                    chat.dispatch(ChatAction::PushRelay("Opening channel".to_owned()));

                    match WebSocket::open(WS_URL) {
                        Ok(socket) => {
                            connection.set(ConnectionState::Connected);
                            chat.dispatch(ChatAction::PushRelay(
                                "Connected to port 7878".to_owned(),
                            ));

                            let (mut write, mut read) = socket.split();
                            let (tx, mut rx) = unbounded::<String>();
                            *outbox.borrow_mut() = Some(tx);

                            let write_loop = async move {
                                while let Some(text) = rx.next().await {
                                    if write.send(Message::Text(text)).await.is_err() {
                                        break;
                                    }
                                }
                            };

                            let read_chat = chat.clone();
                            let read_loop = async move {
                                while let Some(message) = read.next().await {
                                    match message {
                                        Ok(Message::Text(text)) => {
                                            read_chat.dispatch(ChatAction::PushRaw(text));
                                        }
                                        Ok(Message::Bytes(_)) => {}
                                        Err(error) => {
                                            read_chat.dispatch(ChatAction::PushRelay(format!(
                                                "Connection error: {error}"
                                            )));
                                            break;
                                        }
                                    }
                                }
                            };

                            pin_mut!(write_loop);
                            pin_mut!(read_loop);
                            future::select(write_loop, read_loop).await;

                            *outbox.borrow_mut() = None;
                            connection.set(ConnectionState::Disconnected);
                            chat.dispatch(ChatAction::PushRelay("Channel closed".to_owned()));
                        }
                        Err(error) => {
                            *outbox.borrow_mut() = None;
                            connection.set(ConnectionState::Disconnected);
                            chat.dispatch(ChatAction::PushRelay(format!(
                                "Server unreachable: {error}"
                            )));
                        }
                    }
                });

                || ()
            });
        }

        {
            let log_ref = log_ref.clone();
            let message_count = chat.messages.len();

            use_effect_with(message_count, move |_| {
                if let Some(log) = log_ref.cast::<HtmlElement>() {
                    log.set_scroll_top(log.scroll_height());
                }

                || ()
            });
        }

        let connected = *connection == ConnectionState::Connected;
        let connecting = *connection == ConnectionState::Connecting;
        let draft_is_empty = draft.trim().is_empty();
        let status_label = match *connection {
            ConnectionState::Connecting => "connecting",
            ConnectionState::Connected => "online",
            ConnectionState::Disconnected => "offline",
        };
        let status_class = match *connection {
            ConnectionState::Connecting => "is-connecting",
            ConnectionState::Connected => "is-online",
            ConnectionState::Disconnected => "is-offline",
        };
        let peer_count = chat
            .messages
            .iter()
            .filter(|message| message.variant == MessageVariant::Peer)
            .count();

        let oninput = {
            let draft = draft.clone();
            Callback::from(move |event: InputEvent| {
                let input: HtmlInputElement = event.target_unchecked_into();
                draft.set(input.value());
            })
        };

        let onsubmit = {
            let chat = chat.clone();
            let draft = draft.clone();
            let outbox = outbox.clone();

            Callback::from(move |event: SubmitEvent| {
                event.prevent_default();
                let text = draft.trim().to_owned();

                if text.is_empty() {
                    return;
                }

                match outbox.borrow().as_ref() {
                    Some(sender) if sender.unbounded_send(text).is_ok() => {
                        draft.set(String::new());
                    }
                    _ => {
                        chat.dispatch(ChatAction::PushRelay(
                            "Channel is offline; reconnect after the server is running".to_owned(),
                        ));
                    }
                }
            })
        };

        let reconnect = {
            let reconnects = reconnects.clone();
            Callback::from(move |_| reconnects.set(*reconnects + 1))
        };

        let clear_messages = {
            let chat = chat.clone();
            Callback::from(move |_| chat.dispatch(ChatAction::Clear))
        };

        html! {
            <main class="shell">
                <section class="chat-panel" aria-label="Yew websocket chat">
                    <header class="chat-header">
                        <div class="title-block">
                            <p class="eyebrow">{"Tutorial 3 / Yew WebChat"}</p>
                            <h1>{"Nebula Relay"}</h1>
                        </div>
                        <div class={classes!("status-pill", status_class)}>
                            <span class="status-dot" aria-hidden="true"></span>
                            <span>{status_label}</span>
                        </div>
                    </header>

                    <div class="control-strip">
                        <div class="metric">
                            <span class="metric-value">{peer_count}</span>
                            <span class="metric-label">{"messages"}</span>
                        </div>
                        <div class="metric">
                            <span class="metric-value">{"7878"}</span>
                            <span class="metric-label">{"port"}</span>
                        </div>
                        <button
                            class="tool-button"
                            type="button"
                            title="Reconnect"
                            disabled={connecting || connected}
                            onclick={reconnect}
                        >
                            {"Reconnect"}
                        </button>
                        <button
                            class="tool-button"
                            type="button"
                            title="Clear messages"
                            onclick={clear_messages}
                        >
                            {"Clear"}
                        </button>
                    </div>

                    <div class="message-log" ref={log_ref}>
                        if chat.messages.is_empty() {
                            <div class="empty-state">
                                <span class="empty-mark">{"WS"}</span>
                                <p>{"Relay is quiet."}</p>
                            </div>
                        }

                        { for chat.messages.iter().map(render_message) }
                    </div>

                    <form class="composer" onsubmit={onsubmit}>
                        <input
                            class="message-input"
                            type="text"
                            value={(*draft).clone()}
                            {oninput}
                            disabled={!connected}
                            autocomplete="off"
                            placeholder={if connected { "Message the relay" } else { "Waiting for server" }}
                        />
                        <button
                            class="send-button"
                            type="submit"
                            title="Send message"
                            disabled={!connected || draft_is_empty}
                        >
                            <span>{"Send"}</span>
                            <span aria-hidden="true">{"->"}</span>
                        </button>
                    </form>
                </section>
            </main>
        }
    }

    fn parse_incoming_message(raw: &str) -> (String, String, MessageVariant) {
        if raw == "Welcome to chat!" {
            return (
                "relay".to_owned(),
                "Welcome to chat".to_owned(),
                MessageVariant::Relay,
            );
        }

        match raw.split_once(": ") {
            Some((author, body)) => (author.to_owned(), body.to_owned(), MessageVariant::Peer),
            None => ("relay".to_owned(), raw.to_owned(), MessageVariant::Relay),
        }
    }

    fn render_message(message: &ChatMessage) -> Html {
        let variant_class = match message.variant {
            MessageVariant::Peer => "is-peer",
            MessageVariant::Relay => "is-relay",
        };

        html! {
            <article key={message.id.to_string()} class={classes!("message", variant_class)}>
                <span class="message-author">{message.author.as_str()}</span>
                <p class="message-body">{message.body.as_str()}</p>
            </article>
        }
    }

    pub fn run() {
        yew::Renderer::<App>::new().render();
    }
}

#[cfg(target_arch = "wasm32")]
fn main() {
    app::run();
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    println!(
        "Run the frontend with `trunk serve`, or start the chat server with `cargo run --bin server`."
    );
}
