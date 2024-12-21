import { LitElement, html, css } from "lit";

export class App extends LitElement {
    static styles = css`
        a, a:visited {
            padding-right: 15px;
            color: rgba(255, 255, 255, 0.5);
            text-decoration: none;
        }
        
        a:hover {
            color: rgba(255, 255, 255, 0.75);
        }

        .navbar {
            background-color: #343a40;
            padding: 15px;
        }

        .navbar a.active {
            color: white;
        }

        .navbar a.right {
            position: relative;
            float: right;
        }

        .content {
            position: fixed;
            top: 51px;
            bottom: 0px;
            left: 0px;
            right: 0px;
            padding: 15px;
            overflow-y: auto;
        }
    `;

    static get properties() {
        return {
            component: { type: String },
            params: { type: Object },
            reportHash: { type: String },
            mailUid: { type: String },
        };
    }

    constructor() {
        super();
        this.component = "dashboard";
        this.params = {};
        this.reportHash = null;
        this.mailUid = null;
        window.onhashchange = () => this.onHashChange();
        this.onHashChange();
    }

    async onHashChange() {
        let hash = document.location.hash;

        // Split off and parse query params behind route
        const sep = hash.indexOf("?");
        this.params = {};
        if (sep != -1) {
            const param = hash.substring(sep + 1).split("&");
            param.forEach((param) => {
                const keyValue = param.split("=");
                if (keyValue.length === 2) {
                    this.params[keyValue[0]] = keyValue[1];
                }
            });
            hash = hash.substring(0, sep);
        }

        // Parse routes and route parameters
        if (hash == "#/reports") {
            this.component = "reports";
        } else if (hash.startsWith("#/reports/")) {
            this.component = "report";
            this.reportHash = hash.substring(10);
        } else if (hash == "#/mails") {
            this.component = "mails";
        } else if (hash.startsWith("#/mails/")) {
            this.component = "mail";
            this.mailUid = hash.substring(8);
        } else if (hash == "#/about") {
            this.component = "about";
        } else {
            this.component = "dashboard";
        }
    }

    render() {
        let component;
        if (this.component == "reports") {
            component = html`<dmarc-reports .params="${this.params}"></dmarc-reports>`;
        } else if (this.component == "report") {
            component = html`<dmarc-report hash="${this.reportHash}"></dmarc-report>`;
        } else if (this.component == "mails") {
            component = html`<dmarc-mails .params="${this.params}"></dmarc-mails>`;
        } else if (this.component == "mail") {
            component = html`<dmarc-mail uid="${this.mailUid}"></dmarc-mail>`;
        } else if (this.component == "about") {
            component = html`<dmarc-about></dmarc-about>`;
        } else {
            component = html`<dmarc-dashboard></dmarc-dashboard>`;
        }

        return html`
            <div class="navbar">
                <a class="${this.component === "dashboard" ? "active" : ""}" href="#/dashboard">Dashboard</a>
                <a class="${this.component === "mails" ? "active" : ""}" href="#/mails">Mails</a>
                <a class="${this.component === "reports" ? "active" : ""}" href="#/reports">Reports</a>
                <a class="${this.component === "about" ? "active" : ""} right" href="#/about">About</a>
            </div>
            <div class="content">${component}</div>
        `;
    }
}

customElements.define("dmarc-app", App);
