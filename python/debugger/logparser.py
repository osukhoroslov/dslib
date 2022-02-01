import typing as t
from util import Event, Test

class LogParser:
    def __init__(self) -> None:
        self.tests: t.Dict[str, Test] = {}
        self.node_ids: t.Set[str] = {}

    def parse_log_file(self, file_path: str):
        with open(file_path, 'rt') as f:
            last_test_name = ''
            test_event_counter = 0
            for line in f:
                line = line.strip()
                if not line:
                    # for last line (or any empty)
                    continue
                if line.startswith("NODE_IDS"):
                    self.node_ids = set(line.split(":")[1:])
                    continue
                if line.startswith("TEST_BEGIN"):
                    last_test_name = line.split(':', maxsplit=1)[1]
                    self.tests[last_test_name] = Test(last_test_name, [], None, None)
                    continue
                if line.startswith("TEST_END"):
                    test_event_counter = 0
                    _, status, err = line.split(':', maxsplit=2)
                    err = err if err else None
                    self.tests[last_test_name].status = status
                    self.tests[last_test_name].err = err
                    continue

                self.tests[last_test_name].events.append(Event.from_json(line, test_event_counter))
                test_event_counter += 1


        if len(self.tests) == 0 or len(self.node_ids) == 0:
            raise RuntimeError(
                f'Parsed empty data. Tests: {len(self.tests)} '
                f'Nodes: {len(self.node_ids)}'
            )