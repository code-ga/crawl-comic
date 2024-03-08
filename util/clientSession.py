import random
import os.path

proxyList = (
    open(os.path.join(os.path.dirname(__file__), "../proxy/proxy.txt"), "r")
    .read()
    .splitlines()
)


def random_proxy():
    return None
