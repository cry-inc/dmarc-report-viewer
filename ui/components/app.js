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
            reportId: { type: String },
            mailId: {type: String },
        };
    }

    constructor() {
        super();
        this.component = "dashboard";
        this.reportId = null;
        this.mailId = null;
        window.onhashchange = () => this.onHashChange();
        this.onHashChange();
    }

    async onHashChange() {
        const hash = document.location.hash;
        if (hash == "#/reports") {
            this.component = "reports";
        } else if (hash.startsWith("#/reports/")) {
            this.component = "report";
            this.reportId = hash.substring(10);
        } else if (hash == "#/problems") {
            this.component = "problems";
        } else if (hash == "#/mails") {
            this.component = "mails";
        } else if (hash.startsWith("#/mails/")) {
            this.component = "mail";
            this.mailId = hash.substring(8);
        } else {
            this.component = "dashboard";
        }
    }

    render() {
        let component;
        if (this.component == "reports") {
            component = html`<dmarc-reports></dmarc-reports>`;
        } else if (this.component == "report") {
            component = html`<dmarc-report id="${this.reportId}"></dmarc-report>`;
        } else if (this.component == "problems") {
            component = html`<dmarc-problems></dmarc-problems>`;
        } else if (this.component == "mails") {
            component = html`<dmarc-mails></dmarc-mails>`;
        } else if (this.component == "mail") {
            component = html`<dmarc-mail id="${this.mailId}"></dmarc-mail>`;
        } else {
            component = html`<dmarc-dashboard></dmarc-dashboard>`;
        }
        return html`
            <p>
                <a href="#/dashboard">Dashboard</a> |
                <a href="#/reports">Reports</a> |
                <a href="#/mails">Mails</a> |
                <a href="#/problems">Problems</a>
            </p>
            ${component}
        `;
    }
}

customElements.define("dmarc-app", App);
