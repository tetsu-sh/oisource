import { useEffect } from "react";
import {
  DataGrid,
  GridColDef,
  GridValueGetterParams,
  GridToolbar,
} from "@mui/x-data-grid";
import { Box } from "@mui/material";

import { gql, useQuery } from "@apollo/client";
import client from "./api/apolloClient";
import Link from "next/link";

const columns: GridColDef[] = [
  { field: "id", headerName: "ID", width: 120, sortable: true },
  { field: "title", headerName: "Title", width: 800, sortable: true },
  { field: "auther", headerName: "Auther", width: 130, sortable: true },
  {
    field: "media",
    headerName: "Media",
    width: 90,
    sortable: true,
  },
  {
    field: "url",
    headerName: "URL",
    width: 300,
    renderCell: (params) => (
      <a target="_blank" href={`${params.value}`}>
        <u>{params.value}</u>
      </a>
    ),
  },
  {
    field: "createdAt",
    headerName: "created_at",
    type: "datetime",
    width: 160,
    sortable: true,
  },
  {
    field: "crawledAt",
    headerName: "crawled_at",
    type: "datetime",
    width: 160,
    sortable: true,
  },
];

export default function DataTable() {
  const { loading, error, data } = useQuery(gql`
    {
      scan {
        id
        title
        auther
        media
        url
        summary
        createdAt
        crawledAt
      }
    }
  `);
  if (loading) return "Loading...";
  if (error) return `Error! ${error.message}`;
  console.log(data);
  return (
    <div style={{ height: 1000, width: "100%" }}>
      <DataGrid
        components={{ Toolbar: GridToolbar }}
        initialState={{
          filter: {
            filterModel: {
              items: [
                {
                  columnField: "",
                  operatorValue: "",
                  value: "",
                },
              ],
            },
          },
        }}
        rows={data.scan}
        columns={columns}
        pageSize={20}
        rowsPerPageOptions={[10]}
        checkboxSelection
      />
    </div>
  );
}
