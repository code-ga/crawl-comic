from concurrent.futures import ThreadPoolExecutor
import concurrent.futures
import time
import requests

workedProxies = []
TEST_URL = "https://httpbin.org/ip"


def is_bad_proxy(pip):
    global workedProxies
    time.sleep(1)
    print("Testing", pip)
    try:
        response = requests.get(TEST_URL, proxies={"http": pip, "https": pip})
        print(response.json())
    except requests.exceptions.ProxyError as e:
        print("Error code: ", e.code)
        print(f"Bad Proxy: {pip}")
        return e.code
    except Exception as detail:
        print("ERROR:", detail)
        print(f"Bad Proxy: {pip}")
        return True
    print(f"Good Proxy: {pip}")
    workedProxies.append(pip)
    return False


pool = ThreadPoolExecutor(max_workers=100)


def main():

    # two sample proxy IPs
    proxyList = open("testingProxy.txt", "r").read().split("\n")

    futures = []
    for currentProxy in proxyList:
        futures.append(pool.submit(is_bad_proxy, currentProxy))

    for future in concurrent.futures.as_completed(futures):
        print(future.result())
    # on pool completion
    print(workedProxies)
    open("proxy.txt", "w").write("\n".join(workedProxies))
    pool.shutdown()


if __name__ == "__main__":
    main()
