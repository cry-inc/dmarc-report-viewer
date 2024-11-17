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

        .problem {
            border-radius: 3px;
            padding-left: 4px;
            padding-right: 4px;
            color: white;
            background-color: #f00;
        }
    `;

    static properties = {
        mails: { type: Array },
    };

    constructor() {
        super();
        this.mails = [];
    }

    prepareSubject(subject) {
        const regex = /Report Domain: |Report domain: /;
        const prefix_removed = subject.replace(regex, "");
        if (prefix_removed.length < 90) {
            return prefix_removed;
        } else {
            return prefix_removed.substring(0, 75) + "...";
        }
    }

    prepareSize(mail) {
        if (mail.oversized) {
            return html`<span class="problem">${mail.size}</span>`;
        } else {
            return mail.size;
        }
    }

    render() {
        return html`
            <table>
                <tr>
                    <th>Subject</th>
                    <th>Sender</th>
                    <th>Date</th>
                    <th>Size</th>
                </tr>
                ${this.mails.map((mail) =>
                    html`<tr> 
                        <td><a href="#/mails/${mail.uid}">${this.prepareSubject(mail.subject)}</a></td>    
                        <td>${mail.sender}</td>
                        <td>${new Date(mail.date * 1000).toLocaleString()}</td>
                        <td>${this.prepareSize(mail)}</td>
                    </tr>`
                )}
            </table>
        `;
    }
}

customElements.define("dmarc-mail-table", MailTable);
