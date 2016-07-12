import ctypes as ct

class Implementation(object):
    def __init__(self, path):
        self.env = None
        self.lib = ct.CDLL(path)

        self._setup_calls()
        self._setup_environment()

    def __del__(self):
        if self.env is not None:
            self.lib.LFEnvironment_del(self.env)

    def _setup_calls(self):
        self.lib.LFEnvironment_new.restype = ct.c_voidp

    def _setup_environment(self):
        env = self.lib.LFEnvironment_new()
        if env is None:
            raise RuntimeError("Error creating environment")
        self.env = env

