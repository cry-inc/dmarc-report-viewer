import { LitElement, html, css } from "lit";

export class Mail extends LitElement {
    static styles = css`
        table {
            width: 100%;
            margin-bottom: 20px;
        }
    
        th {
            text-align: left;
            background-color: #efefef;
            width: 200px;
        }

        td, th {
            padding-left: 10px;
            padding-right: 10px;
            padding-top: 3px;
            padding-bottom: 3px;
        }

        .bigHeader {
            font-size: 20px;
        }

        .noproblem {
            color: #ccc;
        }

        .problem {
            border-radius: 3px;
            padding-left: 4px;
            padding-right: 4px;
            color: white;
            background-color: #f00;
        }
    `;

    static get properties() {
        return {
            id: { type: String },
            mail: { type: Object, attribute: false }
        };
    }

    constructor() {
        super();
        this.id = null;
        this.mail = null;
    }

    async updated(changedProperties) {
        if (changedProperties.has("id") && changedProperties.id !== this.id && this.id) {
            const response = await fetch("mails/" + this.id);
            this.mail = await response.json();
        }
    }

    renderOversized(oversized) {
        if (oversized) {
            return html`<span class="problem">Yes</span>`;
        } else {
            return html`<span class="noproblem">No</span>`;
        }
    }

    render() {
        if (!this.mail) {
            return html`No mail loaded`;
        }

        return html`
            <table>
                <tr>
                    <th colspan="2" class="bigHeader">Mail</th>
                </tr>
                <tr>
                    <th>UId</th>
                    <td>${this.mail.uid}</td>
                </tr>
                <tr>
                    <th>Size</th>
                    <td>${this.mail.size} bytes</td>
                </tr>
                <tr>
                    <th>Oversized</th>
                    <td>${this.renderOversized(this.mail.oversized)}</td>
                </tr>
                <tr>
                    <th>Date</th>
                    <td>${new Date(this.mail.date * 1000).toLocaleString()}</td>
                </tr>
                <tr>
                    <th>Subject</th>
                    <td>${this.mail.subject}</td>
                </tr>
                <tr>
                    <th>Sender</th>
                    <td>${this.mail.sender}</td>
                </tr>
                <tr>
                    <th>Recipient</th>
                    <td>${this.mail.to}</td>
                </tr>
            </table>
        `;
    }
}

customElements.define("dmarc-mail", Mail);
