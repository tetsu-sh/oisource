import * as React from "react";
import {
  DataGrid,
  GridColDef,
  GridValueGetterParams,
  GridToolbar,
} from "@mui/x-data-grid";

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
    renderCell: (params) => <a href={`${params.value}`}>{params.value}</a>,
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
        rows={rows}
        columns={columns}
        pageSize={20}
        rowsPerPageOptions={[10]}
        checkboxSelection
      />
    </div>
  );
}

const rows = [
  {
    id: "0697cc827efa89e5d93e",
    title:
      "Docker初心者が、Nginxのログを fluentd + elasticsearch + kibana で可視化してみた",
    auther: "",
    media: "qiita",
    url: "https://qiita.com/zgmf_mbfp03/items/0697cc827efa89e5d93e",
    summary: "",
    createdAt: "2018-06-10 02:19:29",
    crawledAt: "2022-12-19 14:31:33",
  },
  {
    id: "0dc64a6e0200ba748110",
    title:
      "CIツールでecspressoを使いつつAWS CodePipelineの承認フローを通してECSへデプロイする",
    auther: "",
    media: "qiita",
    url: "https://qiita.com/momotaro98/items/0dc64a6e0200ba748110",
    summary: "",
    createdAt: "2021-01-03 15:31:12",
    crawledAt: "2022-12-19 14:31:33",
  },
  {
    id: "24728936ee6f6674ae37",
    title:
      "引っ越しすることになったので機械学習を使って全力で自分の住む家を決めようとした話",
    auther: "",
    media: "qiita",
    url: "https://qiita.com/pao2/items/24728936ee6f6674ae37",
    summary: "",
    createdAt: "2020-12-11 00:31:55",
    crawledAt: "2022-12-19 14:31:33",
  },
  {
    id: "3a834de6e2bb8fac7559",
    title: "医療コンペで優勝した話",
    auther: "",
    media: "qiita",
    url: "https://qiita.com/chizuchizu/items/3a834de6e2bb8fac7559",
    summary: "",
    createdAt: "2021-03-30 00:21:08",
    crawledAt: "2022-12-19 14:31:33",
  },
  {
    id: "59d683992ee33de857e9",
    title: "爆速python-fire",
    auther: "",
    media: "qiita",
    url: "https://qiita.com/KtheS/items/59d683992ee33de857e9",
    summary: "",
    createdAt: "2020-06-10 01:34:37",
    crawledAt: "2022-12-19 14:31:33",
  },
  {
    id: "6c0c73a1e79644ebbb1a",
    title: "すべての開発者へ。すごいGitHubリポジトリ10選",
    auther: "",
    media: "qiita",
    url: "https://qiita.com/baby-degu/items/6c0c73a1e79644ebbb1a",
    summary: "",
    createdAt: "2021-05-03 07:05:32",
    crawledAt: "2022-12-19 14:31:33",
  },
  {
    id: "6f7fb1c206f77716ee2a",
    title: "【SRE Next 2020】発表資料まとめ",
    auther: "",
    media: "qiita",
    url: "https://qiita.com/Hassan/items/6f7fb1c206f77716ee2a",
    summary: "",
    createdAt: "2020-01-25 15:32:29",
    crawledAt: "2022-12-19 14:31:33",
  },
  {
    id: "741fcf0f40dd989ee4f8",
    title: "要件定義～システム設計ができる人材になれる記事",
    auther: "",
    media: "qiita",
    url: "https://qiita.com/Saku731/items/741fcf0f40dd989ee4f8",
    summary: "",
    createdAt: "2020-01-11 09:04:24",
    crawledAt: "2022-12-19 14:31:33",
  },
  {
    id: "7b57835cb76e70dd0fc4",
    title: "【JavaScript】var / let / const を本気で使い分けてみた",
    auther: "",
    media: "qiita",
    url: "https://qiita.com/cheez921/items/7b57835cb76e70dd0fc4",
    summary: "",
    createdAt: "2020-09-06 15:03:27",
    crawledAt: "2022-12-19 14:31:33",
  },
  {
    id: "9271a225a8901fa72796",
    title: "VS Code Remote ContainersとPylanceで快適Python環境を構築する方法",
    auther: "",
    media: "qiita",
    url: "https://qiita.com/goto-yuki/items/9271a225a8901fa72796",
    summary: "",
    createdAt: "2021-02-13 21:53:13",
    crawledAt: "2022-12-19 14:31:33",
  },
  {
    id: "93fef037a787318e7067",
    title: "SOLID原則について簡単に書く",
    auther: "",
    media: "qiita",
    url: "https://qiita.com/yui_mop/items/93fef037a787318e7067",
    summary: "",
    createdAt: "2018-12-08 16:42:44",
    crawledAt: "2022-12-19 14:31:33",
  },
  {
    id: "9de86c6cde72060ef0d5",
    title:
      "(随時更新)メンバー30人以下くらいの副業もいるチームの社内セキュリティについて",
    auther: "",
    media: "qiita",
    url: "https://qiita.com/ku_suke/items/9de86c6cde72060ef0d5",
    summary: "",
    createdAt: "2021-10-23 00:44:11",
    crawledAt: "2022-12-19 14:31:33",
  },
  {
    id: "a762b1bc0f192a55eae8",
    title: "【PythonのORM】SQLAlchemyで基本的なSQLクエリまとめ",
    auther: "",
    media: "qiita",
    url: "https://qiita.com/tomo0/items/a762b1bc0f192a55eae8",
    summary: "",
    createdAt: "2017-11-18 15:46:30",
    crawledAt: "2022-12-19 14:31:33",
  },
  {
    id: "c2294b6a40c935ab2ec7",
    title:
      "社内で毎週公開してるセキュリティコンテンツ3ヶ月分を公開します【2020年4月〜7月】",
    auther: "",
    media: "qiita",
    url: "https://qiita.com/minakawa-daiki/items/c2294b6a40c935ab2ec7",
    summary: "",
    createdAt: "2020-07-22 17:10:01",
    crawledAt: "2022-12-19 14:31:33",
  },
  {
    id: "ceeeb7329a4fdc566546",
    title: "転職したらKubernetesだった件",
    auther: "",
    media: "qiita",
    url: "https://qiita.com/yuanying/items/ceeeb7329a4fdc566546",
    summary: "",
    createdAt: "2020-05-26 10:53:19",
    crawledAt: "2022-12-19 14:31:33",
  },
  {
    id: "d210ddc2cb1bfeea9338",
    title:
      "「実践ドメイン駆動設計」を読んだので、実際にDDDで設計して作ってみた！",
    auther: "",
    media: "qiita",
    url: "https://qiita.com/APPLE4869/items/d210ddc2cb1bfeea9338",
    summary: "",
    createdAt: "2018-12-17 11:20:33",
    crawledAt: "2022-12-19 14:31:33",
  },
  {
    id: "d2b46c817d97f330d91d",
    title: "ログ収集系の個人的まとめ",
    auther: "",
    media: "qiita",
    url: "https://qiita.com/zuzu0301/items/d2b46c817d97f330d91d",
    summary: "",
    createdAt: "2017-08-04 21:37:21",
    crawledAt: "2022-12-19 14:31:33",
  },
  {
    id: "d38eeef9f0af5a4a87da",
    title: "知らないと恥ずかしいコードレビューで指摘されがちなポイント14選",
    auther: "",
    media: "qiita",
    url: "https://qiita.com/ouauai/items/d38eeef9f0af5a4a87da",
    summary: "",
    createdAt: "2022-12-13 09:13:30",
    crawledAt: "2022-12-19 14:31:33",
  },
  {
    id: "dbb168b14d1008396a0d",
    title:
      "【初アプリ】未経験がFlutterで肉牛繁殖農家のためのアプリを作ってみた",
    auther: "",
    media: "qiita",
    url: "https://qiita.com/hara_taku_/items/dbb168b14d1008396a0d",
    summary: "",
    createdAt: "2020-06-30 11:25:34",
    crawledAt: "2022-12-19 14:31:33",
  },
  {
    id: "e7196ba496e59bb2aa28",
    title: "エンジニアの情報収集法まとめ",
    auther: "",
    media: "qiita",
    url: "https://qiita.com/nesheep5/items/e7196ba496e59bb2aa28",
    summary: "",
    createdAt: "2015-06-01 01:10:02",
    crawledAt: "2022-12-19 14:31:33",
  },
];
