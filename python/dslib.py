from __future__ import annotations
import abc
import json
import random
from typing import Any, List, Dict, Tuple


class Message:
    def __init__(self, message_type: str, data: Dict[str, Any]):
        self._type = message_type
        self._data = data

    @property
    def type(self) -> str:
        return self._type

    def __getitem__(self, key: str) -> Any:
        return self._data[key]

    def __setitem__(self, key: str, value: Any):
        self._data[key] = value

    def remove(self, key: str):
        self._data.pop(key, None)

    @staticmethod
    def from_json(message_type: str, json_str: str) -> Message:
        return Message(message_type, json.loads(json_str))


class Context(object):
    def __init__(self, time: float):
        self._time = time
        self._sent_messages: List[Tuple[str, str, str]] = list()
        self._sent_local_messages: List[tuple[str, str]] = list()
        self._set_timers: List[Tuple[str, float]] = list()
        self._canceled_timers: List[str] = list()

    def send(self, msg: Message, to: str):
        self._sent_messages.append((msg.type, json.dumps(msg._data), to))

    def send_local(self, msg: Message):
        self._sent_local_messages.append((msg.type, json.dumps(msg._data)))

    def set_timer(self, timer_id: str, delay: float):
        self._set_timers.append((timer_id, delay))
    
    def cancel_timer(self, timer_id: str):
        self._canceled_timers.append(timer_id)

    def time(self) -> float:
        return self._time

    def rand(self) -> float:
        # TODO: use global random initialized with simulation seed
        return random.uniform(0, 1)


class Node:
    @abc.abstractmethod
    def on_local_message(self, msg: Message, ctx: Context):
        pass

    @abc.abstractmethod
    def on_message(self, msg: Message, sender: str, ctx: Context):
        pass

    @abc.abstractmethod
    def on_timer(self, timer_id: str, ctx: Context):
        pass
