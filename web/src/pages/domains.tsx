import { useContext, useState, useEffect } from "react";
import { Link as RouterLink, useParams } from "react-router";
import { Plus, ChevronLeft } from "lucide-react";

import { InfoContext } from "..";
import { AuthContext, fetchAuth } from "../auth";
import { Button, Link, Input } from "../components";

type Domain = {
  name: string;
  ipv4: string | null;
  ipv6: string | null;
};

const removeRoot = (d: string) => (d.slice(-1) === "." ? d.slice(0, -1) : d);

const Domains = () => {
  const { token } = useContext(AuthContext);
  const info = useContext(InfoContext);
  const { name } = useParams();
  const [domains, setDomains] = useState<Domain[]>([]);

  // redirect if invalid!
  const selected = domains.find((d) => d.name === name);

  useEffect(() => {
    (async () => {
      const res = await fetchAuth("/api/domains", token!);
      setDomains(await res.json());
    })();
  }, []);

  return (
    <div className="space-y-2">
      {selected ? (
        <Edit
          domain={selected}
          onUpdate={(n) =>
            setDomains(domains.map((d) => (d.name === n.name ? n : d)))
          }
          onDelete={() =>
            setDomains(domains.filter((d) => d.name !== selected.name))
          }
        />
      ) : (
        <Overview domains={domains} />
      )}
    </div>
  );
};

const Overview = (props: { domains: Domain[] }) => {
  const info = useContext(InfoContext);

  return (
    <>
      <h2 className="text-2xl font-bold">Domains:</h2>
      {props.domains.map((d) => (
        <RouterLink
          className="bg-ctp-mantle rounded-md p-2 block cursor-pointer hover:bg-ctp-crust"
          key={d.name}
          to={d.name}
        >
          <span className="min-w-2/5 inline-block">
            {d.name}
            <span className="text-ctp-overlay0">
              .{removeRoot(info.dns_zone)}
            </span>
          </span>
          <span className="text-ctp-overlay0 max-sm:hidden">
            {[d.ipv4, d.ipv6].filter((x) => x).join(", ")}
          </span>
        </RouterLink>
      ))}
      {/* have input field shown always*/}
      <Button long>
        <Plus className="inline mr-1" />
        todo
      </Button>
    </>
  );
};

const Edit = (props: {
  domain: Domain;
  onUpdate: (_: Domain) => void;
  onDelete: () => void;
}) => {
  const info = useContext(InfoContext);
  const { token } = useContext(AuthContext);
  const [error, setError] = useState<string | null>(null);

  const Field = (editProps: {
    label: string;
    default: string | null;
    type: string;
  }) => {
    const [loading, setLoading] = useState(false);

    const update = async (e: React.FormEvent<HTMLFormElement>) => {
      e.preventDefault();
      setLoading(true);
      setError(null);

      try {
        const ip = e.currentTarget.ip.value;
        const res = await fetchAuth(
          `/api/domains/${props.domain.name}?type=${editProps.type}`,
          token!,
          { method: "PUT", body: ip },
        );
        if (res.status === 200) {
          props.onUpdate({ ...props.domain, [editProps.type]: ip });
        } else if (res.status === 400) {
          setError("Format error.");
        } else {
          setError("Unknown error.");
        }
      } catch {
        setError("api error");
      }
      setLoading(false);
    };

    return (
      <form className="max-sm:space-y-2 sm:space-x-2" onSubmit={update}>
        <strong>{editProps.label}:</strong>
        <Input type="text" name="ip" defaultValue={editProps.default || ""} />
        <Button disabled={loading} className="max-sm:w-full">
          {loading ? "Updating" : "Update"}
        </Button>
      </form>
    );
  };

  const remove = async () => {
    await fetchAuth(`/api/domains/${props.domain.name}`, token!, {
      method: "DELETE",
    });
    props.onDelete();
  };

  return (
    <>
      <h2 className="text-2xl font-bold">
        {props.domain.name}.{removeRoot(info.dns_zone)}:
      </h2>
      <Field label="IPv4" default={props.domain.ipv4} type="ipv4" />
      <Field label="IPv6" default={props.domain.ipv6} type="ipv6" />
      <Button className="block" color="red" onClick={remove}>
        Delete
      </Button>
      {error && <p className="text-ctp-red">{error}</p>}
      <Link href="/dashboard/domains">
        <ChevronLeft className="inline mr-1" />
        Back
      </Link>
    </>
  );
};

export default Domains;
