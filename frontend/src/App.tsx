import { useEffect, useRef, useState } from "react";
import { Button } from "./components/ui/button";
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "./components/ui/dialog";
import { Input } from "./components/ui/input";
import { Label } from "./components/ui/label";
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "./components/ui/select";

type Message = {
  _type: string;
  data: string;
};

function App() {
  const wsRef = useRef<WebSocket | null>(null);
  const [data, setData] = useState<string>("");

  useEffect(() => {
    const ws = new WebSocket("ws://localhost:3000/ws");
    wsRef.current = ws;

    ws.addEventListener("open", () => {
      console.log("Connection established");
      ws.send("get_data");
    });

    ws.addEventListener("message", (event) => {
      const data: Message = JSON.parse(event.data);
      console.log("Data received: ", data);
      switch (data._type) {
        case "data":
          setData(data.data);
          break;

        default:
          break;
      }
    });

    ws.addEventListener("close", () => {
      console.log("Connection closed");
    });

    ws.addEventListener("error", (error) => {
      console.error("Error: ", error);
    });

    return () => {
      ws.close();
    };
  }, []);

  return (
    <div className="p-4">
      <div className="py-2">
        <h1 className="text-4xl font-bold">Mate - Companion</h1>
      </div>
      <div className="grid">
        <div>
          <Dialog>
            <DialogTrigger asChild>
              <Button>Add bot</Button>
            </DialogTrigger>
            <DialogContent className="sm:max-w-[425px]">
              <DialogHeader>
                <DialogTitle>Add bot</DialogTitle>
                <DialogDescription>
                  Add and connect a bot to the server
                </DialogDescription>
              </DialogHeader>
              <div className="grid gap-4 py-4">
                <div className="grid grid-cols-4 items-center gap-4">
                  <Label htmlFor="username" className="text-right">
                    Username
                  </Label>
                  <Input
                    id="username"
                    placeholder="cloei"
                    className="col-span-3"
                  />
                </div>
                <div className="grid grid-cols-4 items-center gap-4">
                  <Label htmlFor="password" className="text-right">
                    Password
                  </Label>
                  <Input
                    id="password"
                    placeholder="123123"
                    className="col-span-3"
                  />
                </div>
                <div className="grid grid-cols-4 items-center gap-4">
                  <Label htmlFor="token" className="text-right">
                    Token
                  </Label>
                  <Input
                    id="token"
                    placeholder="adnudiiem"
                    className="col-span-3"
                  />
                </div>
                <div className="grid grid-cols-4 items-center gap-4">
                  <Label htmlFor="method" className="text-right">
                    Login method
                  </Label>
                  <Select defaultValue="3">
                    <SelectTrigger className="col-span-3">
                      <SelectValue placeholder="Select a login method" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectGroup>
                        <SelectItem value="3">Legacy</SelectItem>
                        <SelectItem value="2">Google</SelectItem>
                        <SelectItem value="1">Apple</SelectItem>
                        <SelectItem value="0">Ubisoft Connect</SelectItem>
                      </SelectGroup>
                    </SelectContent>
                  </Select>
                </div>
              </div>
              <DialogFooter>
                <DialogClose asChild>
                  <Button type="submit">Submit</Button>
                </DialogClose>
              </DialogFooter>
            </DialogContent>
          </Dialog>
        </div>
      </div>
    </div>
  );
}

export default App;
