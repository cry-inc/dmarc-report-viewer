import { LitElement, html, css } from "lit";

export class MailTable extends LitElement {
    static styles = css`
        table {
            width: 100%;
        }

        th {
            text-align: left;
            background-color: #efefef;
        }

        td, th {
            padding-left: 10px;
            padding-right: 10px;
            padding-top: 3px;
            padding-bottom: 3px;
        }

        tr:hover {
            background-color: #f4f4f4;
        }
    `;

    static properties = {
        mails: { type: Array },
    };

    constructor() {
        super();
        this.mails = [];
    }

    render() {
        return html`
            <table>
                <tr> 
                    <th>Sender</th>
                    <th>Recipient</th>
                    <th>Date</th>
                    <th>Size</th>
                    <th>Subject</th>
                </tr>
                ${this.mails.map((mail) =>
                    html`<tr>
                        <td>${mail.sender}</td>
                        <td>${mail.to}</td>
                        <td>${new Date(mail.date * 1000).toLocaleString()}</td>
                        <td>${mail.size}</td>
                        <td>${mail.subject.length < 90 ? mail.subject : mail.subject.substring(0, 90) + "..."}</td>
                    </tr>`
                )}
            </table>
        `;
    }
}

customElements.define("dmarc-mail-table", MailTable);
