import { LitElement, html, css } from "lit";

export class Mails extends LitElement {
    static properties = {
        mails: { type: Array },
    };

    constructor() {
        super();
        this.mails = [];
        this.updateMails();
    }

    async updateMails() {
        const mailsResponse = await fetch("mails");
        this.mails = await mailsResponse.json();
        this.mails.sort((a, b) => b.date - a.date);
    }

    render() {
        return html`<dmarc-mail-table .mails="${this.mails}"></dmarc-mail-table>`;
    }
}

customElements.define("dmarc-mails", Mails);
