import { LitElement, html, css } from "lit";

export class App extends LitElement {
    static styles = css`
        :host {
            font-family: sans-serif;
            font-size: 16px;
        }

        a {
            color: rgb(14, 117, 212);
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
            <p>
                <a href="#/dashboard">Dashboard</a> |
                <a href="#/mails">Mails</a> |
                <a href="#/reports">Reports</a> |
                <a href="#/about">About</a>
            </p>
            ${component}
        `;
    }
}

customElements.define("dmarc-app", App);
