from dslib import Context, Message, Node


class Sender(Node):
    def __init__(self, node_id: str, receiver_id: str):
        self._id = node_id
        self._receiver = receiver_id

    def on_local_message(self, msg: Message, ctx: Context):
        # receive info for delivery from local user
        if msg.type == 'INFO-1':
            ctx.send(msg, "receiver")
        elif msg.type == 'INFO-2':
            # deliver this info at least once
            pass
        elif msg.type == 'INFO-3':
            # deliver this info exactly once
            pass
        elif msg.type == 'INFO-4':
            # deliver these info exactly once and keeping their order
            pass

    def on_message(self, msg: Message, sender: str, ctx: Context):
        # process messages from receiver here
        pass

    def on_timer(self, timer_id: str, ctx: Context):
        # process fired timers here
        pass


class Receiver(Node):
    def __init__(self, node_id: str):
        self._id = node_id
        self._last_message = None

    def on_local_message(self, msg: Message, ctx: Context):
        # not used in this task
        pass

    def on_message(self, msg: Message, sender: str, ctx: Context):
        if msg.type == 'INFO-1':
            if self._last_message is None or self._last_message != msg["info"]:
                ctx.send_local(msg)
            self._last_message = msg["info"]

    def on_timer(self, timer_id: str, ctx: Context):
        # process fired timers here
        pass
