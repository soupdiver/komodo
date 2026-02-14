import { DataTable, SortableHeader } from "@ui/data-table";
import { Types } from "komodo_client";
import { TableTags } from "@components/tags";
import { ResourceLink } from "../common";
import { useSelectedResources } from "@lib/hooks";
import { DeployTraefikButton } from "./deploy-traefik";

export const IngressInstanceTable = ({
  instances,
}: {
  instances: Types.IngressInstanceListItem[];
}) => {
  const [_, setSelectedResources] = useSelectedResources("IngressInstance");
  return (
    <DataTable
      tableKey="ingress-instances"
      data={instances}
      selectOptions={{
        selectKey: ({ name }) => name,
        onSelect: setSelectedResources,
      }}
      columns={[
        {
          accessorKey: "name",
          header: ({ column }) => (
            <SortableHeader column={column} title="Name" />
          ),
          cell: ({ row }) => (
            <ResourceLink type="IngressInstance" id={row.original.id} />
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
        {
          header: "Actions",
          cell: ({ row }) => <DeployTraefikButton id={row.original.id} />,
        },
      ]}
    />
  );
};
