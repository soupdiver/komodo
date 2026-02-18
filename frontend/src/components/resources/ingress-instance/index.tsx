import { useRead, useUser, useLocalStorage, usePermissions, useWrite } from "@lib/hooks";
import { RequiredResourceComponents } from "@types";
import { Network } from "lucide-react";
import { Link } from "react-router-dom";
import { Card, CardDescription, CardHeader, CardTitle } from "@ui/card";
import { DeleteResource, NewResource, ResourcePageHeader } from "../common";
import { IngressInstanceTable } from "./table";
import { Types } from "komodo_client";
import { DeployTraefikButton } from "./deploy-traefik";
import { Config } from "@components/config";
import { ConfigItem } from "@components/config/util";
import { ResourceLink, ResourceSelector } from "@components/resources/common";

const useIngressInstance = (id?: string) =>
  useRead("ListIngressInstances", {}).data?.find((d) => d.id === id);

const IngressInstanceConfig = ({ id }: { id: string }) => {
  const { canWrite } = usePermissions({ type: "IngressInstance", id });
  const config = useRead("GetIngressInstance", { ingress_instance: id }).data?.config;
  const global_disabled = useRead("GetCoreInfo", {}).data?.ui_write_disabled ?? false;
  const { mutateAsync } = useWrite("UpdateIngressInstance");
  const [update, set] = useLocalStorage<Partial<Types.IngressInstanceConfig>>(
    `ingress-instance-${id}-update-v1`,
    {}
  );

  if (!config) return null;
  const disabled = global_disabled || !canWrite;

  return (
    <Config
      disabled={disabled}
      original={config}
      update={update}
      set={set}
      onSave={async () => {
        await mutateAsync({ id, config: update });
      }}
      components={{
        "": [
          {
            label: "Enabled",
            labelHidden: true,
            components: {
              enabled: {
                boldLabel: true,
                description: "Whether to generate Traefik routing config for this ingress instance.",
              },
            },
          },
          {
            label: "Default Instance",
            labelHidden: true,
            components: {
              is_default: {
                boldLabel: true,
                description:
                  "Use as the default instance for containers without an explicit ingress instance label. Only one instance can be default.",
              },
            },
          },
          {
            label: "Server",
            labelHidden: true,
            components: {
              server_id: (server_id, set) => {
                return (
                  <ConfigItem
                    label={
                      server_id ? (
                        <div className="flex gap-3 text-lg font-bold">
                          Server:
                          <ResourceLink type="Server" id={server_id} />
                        </div>
                      ) : (
                        "Select Server"
                      )
                    }
                    description='The server this Traefik runs on. When a target container is on the same server, backend URLs use "host.docker.internal" instead of the server address.'
                  >
                    <ResourceSelector
                      type="Server"
                      selected={server_id}
                      onSelect={(server_id) => set({ server_id })}
                      disabled={disabled}
                      align="start"
                    />
                  </ConfigItem>
                );
              },
            },
          },
        ],
      }}
    />
  );
};

export const IngressInstanceComponents: RequiredResourceComponents = {
  list_item: (id) => useIngressInstance(id),
  resource_links: () => undefined,

  Description: () => <>Manage Traefik ingress instances and dynamic routing.</>,

  GroupActions: () => null,

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
    RouteCount: ({ id }) => {
      const instance = useIngressInstance(id);
      return (
        <div>Routes: {instance?.info.route_count ?? 0}</div>
      );
    },
  },

  Actions: {
    DeployTraefik: ({ id }) => <DeployTraefikButton id={id} />,
  },

  Page: {},

  Config: ({ id }) => <IngressInstanceConfig id={id} />,

  DangerZone: ({ id }) => <DeleteResource type="IngressInstance" id={id} />,

  ResourcePageHeader: ({ id }) => {
    const instance = useIngressInstance(id);
    return (
      <ResourcePageHeader
        intent="None"
        icon={<Network className="w-8" />}
        type="IngressInstance"
        id={id}
        resource={instance}
        state={instance?.info.enabled ? "Enabled" : "Disabled"}
        status={instance?.info.is_default ? "Default" : undefined}
        hideTemplate={true}
      />
    );
  },
};
