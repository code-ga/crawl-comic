from bs4 import BeautifulSoup


class CustomBeautifulSoup(BeautifulSoup):
    def xpath(self, query: str):
        """
        CANCELLED

        Perform an XPath query on the HTML document.

        Args:
        - query (str): The XPath query string.

        Returns:
        - ResultSet: The result of the XPath query as a ResultSet.
        """

        query = (
            query.strip()[1:]
            .replace("/", ">")
            .replace("[", ":nth-child(")
            .replace("]", ")")
        )
        print(query)
        return self.select(query)
