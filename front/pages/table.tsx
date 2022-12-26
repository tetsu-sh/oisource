import { useEffect } from "react";
import {
  DataGrid,
  GridColDef,
  GridValueGetterParams,
  GridToolbar,
} from "@mui/x-data-grid";
import { Box, Button } from "@mui/material";
import { AccessAlarm } from "@mui/icons-material";

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
    field: "summary",
    headerName: "Summary",
    width: 500,
    sortable: true,
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

function full_crawl() {
  const { loading, error, data } = useQuery(gql`
    {
      fullCrawlAndStore {
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
}

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
  function is_latest() {
    const { loading, error, data } = useQuery(gql`
      {
        isLatest {
          
        }
      }
    `);
    console.log(data);
  }
  function update() {}

  return (
    <div style={{ height: 1000, width: "100%" }}>
      <Button variant="contained" onClick={update}>
        update
      </Button>
      <Button variant="contained" onClick={full_crawl}>
        full crawl
      </Button>
      <Button variant="contained" onClick={is_latest}>
        is latest
      </Button>
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
