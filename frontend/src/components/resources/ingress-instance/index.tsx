import { useRead, useUser } from "@lib/hooks";
import { RequiredResourceComponents } from "@types";
import { Network } from "lucide-react";
import { Link } from "react-router-dom";
import { Card, CardDescription, CardHeader, CardTitle } from "@ui/card";
import { DeleteResource, NewResource, ResourcePageHeader } from "../common";
import { IngressInstanceTable } from "./table";
import { Types } from "komodo_client";
import { GroupActions } from "@components/group-actions";

const useIngressInstance = (id?: string) =>
  useRead("ListIngressInstances", {}).data?.find((d) => d.id === id);

export const IngressInstanceComponents: RequiredResourceComponents = {
  list_item: (id) => useIngressInstance(id),
  resource_links: () => undefined,

  Description: () => <>Manage Traefik ingress instances and dynamic routing.</>,

  GroupActions: () => <GroupActions type="IngressInstance" actions={[]} />,

  Dashboard: () => {
    const instances_count = useRead("ListIngressInstances", {}).data?.length;
    return (
      <Link to="/ingress-instances/" className="w-full">
        <Card className="hover:bg-accent/50 transition-colors cursor-pointer">
          <CardHeader>
            <div className="flex justify-between">
              <div>
                <CardTitle>Ingress Instances</CardTitle>
                <CardDescription>{instances_count} Total</CardDescription>
              </div>
              <Network className="w-4 h-4" />
            </div>
          </CardHeader>
        </Card>
      </Link>
    );
  },

  New: () => {
    const is_admin = useUser().data?.admin;
    return is_admin && <NewResource type="IngressInstance" />;
  },

  Table: ({ resources }) => (
    <IngressInstanceTable instances={resources as Types.IngressInstanceListItem[]} />
  ),

  Icon: () => <Network className="w-4 h-4" />,
  BigIcon: () => <Network className="w-8 h-8" />,

  State: () => null,
  Status: {},

  Info: {
    Enabled: ({ id }) => {
      const instance = useIngressInstance(id);
      return (
        <div>Enabled: {instance?.info.enabled ? "Yes" : "No"}</div>
      );
    },
    Default: ({ id }) => {
      const instance = useIngressInstance(id);
      return (
        <div>Default: {instance?.info.is_default ? "Yes" : "No"}</div>
      );
    },
  },

  Actions: {},

  Page: {},

  Config: () => <div className="p-4">Config form coming soon</div>,

  DangerZone: ({ id }) => <DeleteResource type="IngressInstance" id={id} />,

  ResourcePageHeader: ({ id }) => (
    <ResourcePageHeader type="IngressInstance" id={id} />
  ),
};
