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
            uid: { type: String },
            mail: { type: Object, attribute: false },
            reports: { type: Array, attribute: false }
        };
    }

    constructor() {
        super();
        this.uid = null;
        this.mail = null;
        this.reports = [];
    }

    async updated(changedProperties) {
        if (changedProperties.has("uid") && changedProperties.uid !== this.uid && this.uid) {
            const mailsResponse = await fetch("mails/" + this.uid);
            this.mail = await mailsResponse.json();
            const reportsResponse = await fetch("reports?uid=" + this.uid);
            this.reports = await reportsResponse.json();
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
            <h2>Mail</h2>
            <table>
                <tr>
                    <th>UID</th>
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
                    <td>
                        <a href="#/mails?sender=${encodeURIComponent(this.mail.sender)}">
                            ${this.mail.sender}
                        </a>
                    </td>
                </tr>
                <tr>
                    <th>Recipient</th>
                    <td>${this.mail.to}</td>
                </tr>
            </table>

            <h3>Reports extracted from this Mail</h3>
            <dmarc-report-table .reports="${this.reports}"></dmarc-report-table>
        `;
    }
}

customElements.define("dmarc-mail", Mail);
