import { LitElement, html, css } from "lit";
import { globalStyle } from "../style.js";

export class App extends LitElement {
    static styles = [globalStyle, css`
        a, a:visited {
            padding-right: 15px;
            color: rgba(255, 255, 255, 0.5);
            text-decoration: none;
        }

        a:hover {
            color: rgba(255, 255, 255, 0.75);
        }

        a.active {
            color: white;
        }

        a.right {
            position: relative;
            float: right;
        }

        nav {
            background-color: #343a40;
            padding: 15px;
        }

        main {
            position: fixed;
            top: 52px;
            bottom: 0px;
            left: 0px;
            right: 0px;
            padding: 15px;
            overflow-y: auto;
        }
    `];

    static get properties() {
        return {
            component: { type: String },
            params: { type: Object },
            reportId: { type: String },
            mailId: { type: String },
        };
    }

    constructor() {
        super();
        this.component = "dashboard";
        this.params = {};
        this.reportId = null;
        this.mailId = null;
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
        if (hash == "#/dmarc-reports") {
            this.component = "dmarc-reports";
        } else if (hash == "#/tlsrpt-reports") {
            this.component = "tlsrpt-reports";
        } else if (hash.startsWith("#/dmarc-reports/")) {
            this.component = "dmarc-report";
            this.reportId = hash.substring(16);
        } else if (hash.startsWith("#/tlsrpt-reports/")) {
            this.component = "tlsrpt-report";
            this.reportId = hash.substring(17);
        } else if (hash == "#/mails") {
            this.component = "mails";
        } else if (hash.startsWith("#/mails/")) {
            this.component = "mail";
            this.mailId = hash.substring(8);
        } else if (hash == "#/about") {
            this.component = "about";
        } else {
            this.component = "dashboard";
        }
    }

    render() {
        let component;
        if (this.component == "dmarc-reports") {
            component = html`<drv-dmarc-reports .params="${this.params}"></drv-dmarc-reports>`;
        } else if (this.component == "tlsrpt-reports") {
            component = html`<drv-tlsrpt-reports .params="${this.params}"></drv-tlsrpt-reports>`;
        } else if (this.component == "dmarc-report") {
            component = html`<drv-dmarc-report id="${this.reportId}"></drv-dmarc-report>`;
        } else if (this.component == "tlsrpt-report") {
            component = html`<drv-tlsrpt-report id="${this.reportId}"></drv-tlsrpt-report>`;
        } else if (this.component == "mails") {
            component = html`<drv-mails .params="${this.params}"></drv-mails>`;
        } else if (this.component == "mail") {
            component = html`<drv-mail id="${this.mailId}"></drv-mail>`;
        } else if (this.component == "about") {
            component = html`<drv-about></drv-about>`;
        } else {
            component = html`<drv-dashboard .params="${this.params}"></drv-dashboard>`;
        }

        return html`
            <nav>
                <a class="${this.component === "dashboard" ? "active" : ""}" href="#/dashboard">Dashboard</a>
                <a class="${this.component === "mails" || this.component === "mail" ? "active" : ""}" href="#/mails">Mails</a>
                <a class="${this.component === "dmarc-reports" || this.component === "dmarc-report" ? "active" : ""}" href="#/dmarc-reports">DMARC<span class="xs-hidden">&nbsp;Reports</span></a>
                <a class="${this.component === "tlsrpt-reports" || this.component === "tlsrpt-report" ? "active" : ""}" href="#/tlsrpt-reports"><span class="xs-hidden">SMTP&nbsp;</span>TLS<span class="xs-hidden">&nbsp;Reports</span></a>
                <a class="xs-hidden ${this.component === "about" ? "active" : ""} right" href="#/about">About</a>
            </nav>
            <main>${component}</main>
        `;
    }
}

customElements.define("drv-app", App);
