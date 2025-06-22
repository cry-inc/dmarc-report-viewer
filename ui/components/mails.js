import { LitElement, html } from "lit";
import { globalStyle } from "../style.js";

export class Mails extends LitElement {
    static styles = [globalStyle];

    static properties = {
        params: { type: Object },
        mails: { type: Array },
    };

    constructor() {
        super();
        this.params = {};
        this.mails = [];
        this.filtered = false;
    }

    updated(changedProperties) {
        if (changedProperties.has("params")) {
            this.updateMails();
        }
    }

    async updateMails() {
        const queryParams = [];
        if (this.params.oversized === "true" || this.params.oversized === "false") {
            queryParams.push("oversized=" + this.params.oversized);
        }
        if (this.params.sender) {
            queryParams.push("sender=" + encodeURIComponent(this.params.sender));
        }
        if (this.params.attachment) {
            queryParams.push("attachment=" + encodeURIComponent(this.params.attachment));
        }
        if (this.params.errors === "true" || this.params.errors === "false") {
            queryParams.push("errors=" + this.params.errors);
        }
        let url = "mails";
        if (queryParams.length > 0) {
            url += "?" + queryParams.join("&");
        }
        const mailsResponse = await fetch(url);
        this.mails = await mailsResponse.json();
        this.mails.sort((a, b) => b.date - a.date);
        this.filtered = this.filtered = queryParams.length > 0;
    }

    render() {
        return html`
            <h1>Mails</h1>
            <div>
                ${this.filtered ?
                    html`Filter active! <a class="ml button" href="#/mails">Show all Mails</a>` :
                    html`Filters: <a class="ml button" href="#/mails?oversized=true">Oversized Mails</a>
                         <a class="button" href="#/mails?attachment=dmarc&oversized=false">With DMARC</a>
                         <a class="button" href="#/mails?attachment=tls&oversized=false">With TLS</a>
                         <a class="button" href="#/mails?attachment=none&oversized=false">Without Files</a>
                         <a class="button" href="#/mails?errors=true">Parsing Errors</a>`
            }
            </div>
            <drv-mail-table .mails="${this.mails}"></drv-mail-table>
        `;
    }
}

customElements.define("drv-mails", Mails);
