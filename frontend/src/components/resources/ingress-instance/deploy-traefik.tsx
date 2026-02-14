import { useState } from "react";
import { useRead, useWrite } from "@lib/hooks";
import { useNavigate } from "react-router-dom";
import { Network } from "lucide-react";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@ui/dialog";
import { Button } from "@ui/button";
import { Input } from "@ui/input";
import { Label } from "@ui/label";
import { Checkbox } from "@ui/checkbox";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@ui/select";

interface Entrypoint {
  name: string;
  port: number;
  enabled: boolean;
}

const buildComposeYaml = (
  instanceName: string,
  coreUrl: string,
  entrypoints: Entrypoint[]
): string => {
  const enabledEntrypoints = entrypoints.filter((e) => e.enabled);

  const commandLines = [
    "      - --api.insecure=true",
    ...enabledEntrypoints.map(
      (e) => `      - --entrypoints.${e.name}.address=:${e.port}`
    ),
    `      - --providers.http.endpoint=${coreUrl}/ingress/${instanceName}/config`,
    "      - --providers.http.pollInterval=5s",
  ];

  const portLines = enabledEntrypoints.map(
    (e) => `      - "${e.port}:${e.port}"`
  );

  return `services:
  traefik:
    image: traefik:v3.4
    command:
${commandLines.join("\n")}
    ports:
${portLines.join("\n")}
    restart: unless-stopped
`;
};

export const DeployTraefikButton = ({ id }: { id: string }) => {
  const instance = useRead("ListIngressInstances", {}).data?.find(
    (d) => d.id === id
  );
  const servers = useRead("ListServers", {}).data || [];
  const navigate = useNavigate();
  const { mutate: createStack, isPending } = useWrite("CreateStack", {
    onSuccess: (stack) => {
      setOpen(false);
      navigate(`/stacks/${stack._id?.$oid}`);
    },
  });

  const [open, setOpen] = useState(false);
  const [selectedServer, setSelectedServer] = useState<string>("");
  const [coreUrl, setCoreUrl] = useState("http://komodo-core:9120");
  const [entrypoints, setEntrypoints] = useState<Entrypoint[]>([
    { name: "web", port: 80, enabled: true },
    { name: "websecure", port: 443, enabled: true },
    { name: "traefik", port: 8080, enabled: false },
  ]);

  const updateEntrypoint = (
    name: string,
    updates: Partial<Entrypoint>
  ) => {
    setEntrypoints((prev) =>
      prev.map((e) => (e.name === name ? { ...e, ...updates } : e))
    );
  };

  const handleCreate = () => {
    if (!instance || !selectedServer) return;

    const yaml = buildComposeYaml(instance.name, coreUrl, entrypoints);
    createStack({
      name: `traefik-${instance.name}`,
      config: {
        file_contents: yaml,
        server_id: selectedServer,
      },
    });
  };

  if (!instance) return null;

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button variant="secondary" className="gap-2">
          <Network className="w-4 h-4" />
          Deploy Traefik
        </Button>
      </DialogTrigger>
      <DialogContent className="max-w-2xl">
        <DialogHeader>
          <DialogTitle>Deploy Traefik for {instance.name}</DialogTitle>
        </DialogHeader>

        <div className="grid gap-4 py-4">
          <div className="grid gap-2">
            <Label htmlFor="server">Server</Label>
            <Select value={selectedServer} onValueChange={setSelectedServer}>
              <SelectTrigger id="server">
                <SelectValue placeholder="Select a server" />
              </SelectTrigger>
              <SelectContent>
                {servers.map((server) => (
                  <SelectItem key={server.id} value={server.id}>
                    {server.name}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          <div className="grid gap-2">
            <Label htmlFor="core-url">Komodo Core URL</Label>
            <Input
              id="core-url"
              value={coreUrl}
              onChange={(e) => setCoreUrl(e.target.value)}
              placeholder="http://komodo-core:9120"
            />
          </div>

          <div className="grid gap-2">
            <Label>Entrypoints</Label>
            <div className="border rounded-md">
              {entrypoints.map((ep) => (
                <div
                  key={ep.name}
                  className="flex items-center gap-3 p-3 border-b last:border-b-0"
                >
                  <Checkbox
                    checked={ep.enabled}
                    onCheckedChange={(checked) =>
                      updateEntrypoint(ep.name, { enabled: !!checked })
                    }
                  />
                  <Label className="w-24 capitalize">{ep.name}</Label>
                  <Input
                    type="number"
                    value={ep.port}
                    onChange={(e) =>
                      updateEntrypoint(ep.name, {
                        port: parseInt(e.target.value) || 0,
                      })
                    }
                    disabled={!ep.enabled}
                    className="w-24"
                  />
                </div>
              ))}
            </div>
          </div>
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => setOpen(false)}>
            Cancel
          </Button>
          <Button
            onClick={handleCreate}
            disabled={!selectedServer || isPending}
          >
            {isPending ? "Creating..." : "Create Stack"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};
