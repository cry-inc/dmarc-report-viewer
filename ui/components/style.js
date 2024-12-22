import { css } from "lit";

export const globalStyle = css`
    a {
        color: rgb(0, 123, 255);
        text-decoration: none;
    }

    a:hover {
        color: rgb(0, 86, 179);
    }

    .badge {
        border-radius: 3px;
        padding-left: 4px;
        padding-right: 4px;
        background-color: #888;
        color: white;
    }

    .badge-negative {
        background-color: #f00;
    }

    .badge-positive {
        background-color: #090;
    }

    .noproblem {
        color: #ccc;
    }

    .help {
        cursor: help;
    }

    table {
        width: 100%;
        margin-top: 15px;
    }

    th {
        text-align: left;
        background-color: #efefef;
    }

    td, th {
        padding-left: 15px;
        padding-right: 15px;
        padding-top: 3px;
        padding-bottom: 3px;
    }

    table:not(.vertical) tr:hover {
        background-color: #f4f4f4;
    }

    table.vertical th {
        width: 175px;
    }

    h1, h2, h3 {
        padding: 0px;
        margin-top: 15px;
        margin-bottom: 15px;
    }

    h1 {
        margin-top: 0px;
    }
`;
