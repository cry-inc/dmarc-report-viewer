# Access MS 365 Mail Access with DavMail IMAP Proxy
This Docker-Compose example shows how to set up DavMail and the DMARC-Report-Viewer
together so it possible to read reports from an MS mail account.

## Instructions
1. You have Docker and Docker-Compose installed
2. Your MS account admin has given his constent
3. You have an app registered (single tenant) in Entra ID
4. You have public client flows enabled (Manage > Authentication > Settings)
5. You have set up delegated API permissions for:
    - IMAP.AccessAsUser.All
    - offline_access
    - openid
    - profile
6. Copy [docker-compose.yml](docker-compose.yml) and [davmail.properties](davmail.properties)
7. Replace the placeholder values in the `davmail.properties` file
8. Replace the placeholder values in the `docker-compose.yml` file
9. Start the containers by running the command `docker compose up`
