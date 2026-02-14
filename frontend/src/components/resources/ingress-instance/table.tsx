import { DataTable, SortableHeader } from "@ui/data-table";
import { Types } from "komodo_client";
import { TableTags } from "@components/tags";

export const IngressInstanceTable = ({
  instances,
}: {
  instances: Types.IngressInstanceListItem[];
}) => {
  return (
    <DataTable
      tableKey="ingress-instances"
      data={instances}
      columns={[
        {
          accessorKey: "name",
          header: ({ column }) => (
            <SortableHeader column={column} title="Name" />
          ),
        },
        {
          header: "Enabled",
          accessorFn: (instance) => (instance.info.enabled ? "Yes" : "No"),
        },
        {
          header: "Default",
          accessorFn: (instance) => (instance.info.is_default ? "Yes" : "No"),
        },
        {
          header: "Routes",
          accessorFn: (instance) => instance.info.route_count.toString(),
        },
        {
          header: "Tags",
          cell: ({ row }) => <TableTags tag_ids={row.original.tags} />,
        },
      ]}
    />
  );
};
