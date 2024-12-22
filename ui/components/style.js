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
`;
