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
import { Tabs, TabsContent, TabsList, TabsTrigger } from "./components/ui/tabs";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "./components/ui/card";
import {
  Pagination,
  PaginationContent,
  PaginationItem,
  PaginationLink,
  PaginationNext,
  PaginationPrevious,
} from "@/components/ui/pagination";
import { Alert, AlertDescription, AlertTitle } from "./components/ui/alert";
import { Code } from "lucide-react";
import { Sprite, Stage } from "@pixi/react";

type Message = {
  _type: string;
  data: string;
};

interface Item {
  id: number;
  flags: number;
  action_type: number;
  material: number;
  name: string;
  texture_file_name: string;
  texture_hash: number;
  cooking_ingredient: number;
  visual_effect: number;
  texture_x: number;
  texture_y: number;
  render_type: number;
  is_stripey_wallpaper: number;
  collision_type: number;
  block_health: number;
  drop_chance: number;
  clothing_type: number;
  rarity: number;
  max_item: number;
  file_name: string;
  file_hash: number;
  audio_volume: number;
  pet_name: string;
  pet_prefix: string;
  pet_suffix: string;
  pet_ability: string;
  seed_base_sprite: number;
  seed_overlay_sprite: number;
  tree_base_sprite: number;
  tree_overlay_sprite: number;
  base_color: number;
  overlay_color: number;
  ingredient: number;
  grow_time: number;
  is_rayman: number;
  extra_options: string;
  texture_path_2: string;
  extra_option2: string;
  punch_option: string;
}

interface ItemDatabase {
  version: number;
  item_count: number;
  items: Record<number, Item>;
}

type Data = {
  game_version: string;
  protocol: string;
  bots: Bot[];
};

type Bot = {
  username: string;
  password: string;
  token: string;
  login_method: string;
};

function App() {
  const wsRef = useRef<WebSocket | null>(null);
  const [data, setData] = useState<Data | null>(null);
  const [itemDatabase, setItemDatabase] = useState<ItemDatabase | null>(null);
  const [search, setSearch] = useState<string>("");
  const [itemPage, setItemPage] = useState<number>(1);
  // Bad practice, I only do this because I'm prototyping
  const usernameRef = useRef<HTMLInputElement | null>(null);
  const passwordRef = useRef<HTMLInputElement | null>(null);
  const tokenRef = useRef<HTMLInputElement | null>(null);
  const methodRef = useRef<string | null>(null);
  const [selectedBot, setSelectedBot] = useState<Bot | null>(null);

  useEffect(() => {
    const ws = new WebSocket("ws://localhost:3000/ws");
    wsRef.current = ws;

    ws.addEventListener("open", () => {
      console.log("Connection established");
      ws.send(
        JSON.stringify({
          _type: "get_data",
        })
      );
    });

    ws.addEventListener("message", (event) => {
      const { _type, data }: Message = JSON.parse(event.data);
      switch (_type) {
        case "data":
          setData(JSON.parse(data));
          break;
        case "item_database": {
          const typedData = data as unknown as ItemDatabase;
          setItemDatabase({
            version: typedData.version,
            item_count: typedData.item_count,
            items: typedData.items,
          });
          break;
        }
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

  const getItemDatabase = () => {
    setSearch("");
    setItemPage(1);
    if (itemDatabase) return;
    wsRef.current?.send(
      JSON.stringify({
        _type: "get_item_database",
      })
    );
  };

  const addBot = () => {
    if (
      !usernameRef.current?.value ||
      !passwordRef.current?.value ||
      !tokenRef.current?.value ||
      !methodRef.current
    )
      return;
    wsRef.current?.send(
      JSON.stringify({
        _type: "add_bot",
        data: {
          username: usernameRef.current.value,
          password: passwordRef.current.value,
          token: tokenRef.current.value,
          login_method: methodRef.current,
        },
      })
    );
  };

  return (
    <div className="p-4 md:p-10 overflow-x-hidden min-h-full">
      <div className="py-2">
        <h1 className="text-4xl font-bold">Mate - Companion</h1>
      </div>
      <Alert>
        <Code className="h-4 w-4" />
        <AlertTitle>Heads up!</AlertTitle>
        <AlertDescription>
          Currently heavily under development, expect bugs and missing features
        </AlertDescription>
      </Alert>
      <Tabs defaultValue="bots" className="mt-2">
        <div className="flex space-x-2">
          <TabsList>
            <TabsTrigger value="bots">Bots</TabsTrigger>
            <TabsTrigger value="database" onClick={getItemDatabase}>
              Item database
            </TabsTrigger>
          </TabsList>
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
                    ref={usernameRef}
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
                    ref={passwordRef}
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
                    ref={tokenRef}
                  />
                </div>
                <div className="grid grid-cols-4 items-center gap-4">
                  <Label htmlFor="method" className="text-right">
                    Login method
                  </Label>
                  <Select
                    defaultValue="LEGACY"
                    onValueChange={(e) => (methodRef.current = e)}
                  >
                    <SelectTrigger className="col-span-3">
                      <SelectValue placeholder="Select a login method" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectGroup>
                        <SelectItem value="LEGACY">Legacy</SelectItem>
                        <SelectItem value="GOOGLE">Google</SelectItem>
                        <SelectItem value="APPLE">Apple</SelectItem>
                        <SelectItem value="UBISOFT">Ubisoft Connect</SelectItem>
                      </SelectGroup>
                    </SelectContent>
                  </Select>
                </div>
              </div>
              <DialogFooter>
                <DialogClose asChild>
                  <Button type="submit" onClick={addBot}>
                    Submit
                  </Button>
                </DialogClose>
              </DialogFooter>
            </DialogContent>
          </Dialog>
        </div>
        <TabsContent value="bots">
          {data && (
            <div className="grid grid-cols-4">
              <div className="space-y-2">
                {data.bots.map((bot) => {
                  return (
                    <Card
                      key={bot.username}
                      onClick={() => setSelectedBot(bot)}
                      className="rounded-none"
                    >
                      <CardHeader>
                        <CardTitle>{bot.username}</CardTitle>
                        <CardDescription>{bot.login_method}</CardDescription>
                      </CardHeader>
                    </Card>
                  );
                })}
              </div>
              <div className="col-span-3">
                {selectedBot && (
                  <Card className="h-full rounded-none">
                    <Tabs className="p-2">
                      <TabsList>
                        <TabsTrigger value="general">General</TabsTrigger>
                        <TabsTrigger value="world">World</TabsTrigger>
                        <TabsTrigger value="console">Console</TabsTrigger>
                      </TabsList>
                      <TabsContent value="general">
                        <CardContent>
                          <p>Username: {selectedBot.username}</p>
                          <p>Password: {selectedBot.password}</p>
                          <p>Token: {selectedBot.token}</p>
                          <p>Login method: {selectedBot.login_method}</p>
                        </CardContent>
                      </TabsContent>
                      <TabsContent value="world">
                        <CardContent>
                          <Stage
                            className="mx-auto"
                            width={(100 * 32) / 4}
                            height={(60 * 32) / 4}
                          >
                            <Sprite image={`/generic_menu.png`} />
                          </Stage>
                          <div className="flex justify-center space-x-4">
                            <p className="text-center">
                              World name :{" "}
                              <span className="text-rose-500">EXIT</span>{" "}
                            </p>
                            <p>Pos ( -:- )</p>
                          </div>
                        </CardContent>
                      </TabsContent>
                    </Tabs>
                  </Card>
                )}
              </div>
            </div>
          )}
        </TabsContent>
        <TabsContent value="database">
          {itemDatabase && (
            <>
              <div className="flex space-x-4">
                <p>Items database version : {itemDatabase.version}</p>
                <p>Items count : {itemDatabase.item_count}</p>
              </div>
              <Input
                placeholder="Search an item"
                className="my-2"
                onChange={(e) => setSearch(e.target.value)}
              />
              {!(search.length > 0) && (
                <Pagination className="my-2">
                  <PaginationContent>
                    <PaginationItem>
                      <PaginationPrevious
                        href="#"
                        onClick={() => setItemPage(Math.max(1, itemPage - 1))}
                      />
                    </PaginationItem>
                    {Array.from(
                      { length: Math.ceil(itemDatabase.item_count / 50) },
                      (_, i) => i + 1
                    )
                      .slice(
                        Math.max(
                          0,
                          Math.min(
                            itemPage - 1,
                            Math.ceil(itemDatabase.item_count / 50) - 4
                          )
                        ),
                        Math.min(
                          itemPage + 3,
                          Math.ceil(itemDatabase.item_count / 50)
                        )
                      )
                      .map((page) => (
                        <PaginationItem key={page}>
                          <PaginationLink
                            href="#"
                            onClick={() => setItemPage(page)}
                            isActive={page === itemPage}
                          >
                            {page}
                          </PaginationLink>
                        </PaginationItem>
                      ))}
                    <PaginationItem>
                      <PaginationNext
                        href="#"
                        onClick={() =>
                          setItemPage(
                            Math.min(
                              Math.ceil(itemDatabase.item_count / 50),
                              itemPage + 1
                            )
                          )
                        }
                      />
                    </PaginationItem>
                  </PaginationContent>
                </Pagination>
              )}
              <div className="grid grid-cols-2 md:grid-cols-10">
                {search.length > 0 ? (
                  <>
                    {Object.values(itemDatabase.items)
                      .filter((item) =>
                        item.name.toLowerCase().includes(search.toLowerCase())
                      )
                      .map((item, i) => (
                        <Dialog key={i}>
                          <DialogTrigger asChild>
                            <Card className="rounded-none cursor-pointer">
                              <CardHeader>
                                <CardTitle>{item.name}</CardTitle>
                                <CardDescription>
                                  ID : {item.id}
                                </CardDescription>
                              </CardHeader>
                              <CardContent>
                                <div
                                  style={{
                                    backgroundImage: `url(/${item.texture_file_name.replace(
                                      ".rttex",
                                      ".png"
                                    )})`,
                                    backgroundPosition: `${
                                      item.texture_x * -32
                                    }px ${item.texture_y * -32}px`,
                                    width: "32px",
                                    height: "32px",
                                  }}
                                ></div>
                              </CardContent>
                            </Card>
                          </DialogTrigger>
                          <DialogContent className="sm:max-w-[425px]">
                            <DialogHeader>
                              <DialogTitle>
                                {item.name}{" "}
                                <span className="text-xs font-mono">
                                  ( ID : {item.id} )
                                </span>
                              </DialogTitle>
                            </DialogHeader>
                            <div className="grid grid-cols-2 gap-4">
                              <div>
                                <p>Name: {item.name}</p>
                                <p>ID: {item.id}</p>
                                <p>Flags: {item.flags}</p>
                                <p>Action Type: {item.action_type}</p>
                                <p>Material: {item.material}</p>
                                <p>
                                  Texture File Name: {item.texture_file_name}
                                </p>
                                <p>Texture Hash: {item.texture_hash}</p>
                                <p>
                                  Cooking Ingredient: {item.cooking_ingredient}
                                </p>
                                <p>Visual Effect: {item.visual_effect}</p>
                                <p>Texture X: {item.texture_x}</p>
                                <p>Texture Y: {item.texture_y}</p>
                                <p>Render Type: {item.render_type}</p>
                                <p>
                                  Is Stripey Wallpaper:{" "}
                                  {item.is_stripey_wallpaper}
                                </p>
                                <p>Collision Type: {item.collision_type}</p>
                                <p>Block Health: {item.block_health}</p>
                                <p>Drop Chance: {item.drop_chance}</p>
                                <p>Clothing Type: {item.clothing_type}</p>
                                <p>Rarity: {item.rarity}</p>
                                <p>Max Item: {item.max_item}</p>
                              </div>
                              <div>
                                <p>File Name: {item.file_name}</p>
                                <p>File Hash: {item.file_hash}</p>
                                <p>Audio Volume: {item.audio_volume}</p>
                                <p>Pet Name: {item.pet_name}</p>
                                <p>Pet Prefix: {item.pet_prefix}</p>
                                <p>Pet Suffix: {item.pet_suffix}</p>
                                <p>Pet Ability: {item.pet_ability}</p>
                                <p>Seed Base Sprite: {item.seed_base_sprite}</p>
                                <p>
                                  Seed Overlay Sprite:{" "}
                                  {item.seed_overlay_sprite}
                                </p>
                                <p>Tree Base Sprite: {item.tree_base_sprite}</p>
                                <p>
                                  Tree Overlay Sprite:{" "}
                                  {item.tree_overlay_sprite}
                                </p>
                                <p>Base Color: {item.base_color}</p>
                                <p>Overlay Color: {item.overlay_color}</p>
                                <p>Ingredient: {item.ingredient}</p>
                                <p>Grow Time: {item.grow_time}</p>
                                <p>Is Rayman: {item.is_rayman}</p>
                                <p>Extra Options: {item.extra_options}</p>
                                <p>Texture Path 2: {item.texture_path_2}</p>
                                <p>Extra Option 2: {item.extra_option2}</p>
                                <p>Punch Option: {item.punch_option}</p>
                              </div>
                            </div>
                          </DialogContent>
                        </Dialog>
                      ))}
                  </>
                ) : (
                  <>
                    {Object.values(itemDatabase.items)
                      .slice((itemPage - 1) * 50, itemPage * 50)
                      .map((item, i) => (
                        <Dialog key={i}>
                          <DialogTrigger asChild>
                            <Card className="rounded-none cursor-pointer">
                              <CardHeader>
                                <CardTitle>{item.name}</CardTitle>
                                <CardDescription>
                                  ID : {item.id}
                                </CardDescription>
                              </CardHeader>
                              <CardContent>
                                <div
                                  style={{
                                    backgroundImage: `url(/${item.texture_file_name.replace(
                                      ".rttex",
                                      ".png"
                                    )})`,
                                    backgroundPosition: `${
                                      item.texture_x * -32
                                    }px ${item.texture_y * -32}px`,
                                    width: "32px",
                                    height: "32px",
                                  }}
                                ></div>
                              </CardContent>
                            </Card>
                          </DialogTrigger>
                          <DialogContent className="sm:max-w-[425px]">
                            <DialogHeader>
                              <DialogTitle>
                                {item.name}{" "}
                                <span className="text-xs font-mono">
                                  ( ID : {item.id} )
                                </span>
                              </DialogTitle>
                            </DialogHeader>
                            <div className="grid grid-cols-2 gap-4">
                              <div>
                                <p>Name: {item.name}</p>
                                <p>ID: {item.id}</p>
                                <p>Flags: {item.flags}</p>
                                <p>Action Type: {item.action_type}</p>
                                <p>Material: {item.material}</p>
                                <p>
                                  Texture File Name: {item.texture_file_name}
                                </p>
                                <p>Texture Hash: {item.texture_hash}</p>
                                <p>
                                  Cooking Ingredient: {item.cooking_ingredient}
                                </p>
                                <p>Visual Effect: {item.visual_effect}</p>
                                <p>Texture X: {item.texture_x}</p>
                                <p>Texture Y: {item.texture_y}</p>
                                <p>Render Type: {item.render_type}</p>
                                <p>
                                  Is Stripey Wallpaper:{" "}
                                  {item.is_stripey_wallpaper}
                                </p>
                                <p>Collision Type: {item.collision_type}</p>
                                <p>Block Health: {item.block_health}</p>
                                <p>Drop Chance: {item.drop_chance}</p>
                                <p>Clothing Type: {item.clothing_type}</p>
                                <p>Rarity: {item.rarity}</p>
                                <p>Max Item: {item.max_item}</p>
                              </div>
                              <div>
                                <p>File Name: {item.file_name}</p>
                                <p>File Hash: {item.file_hash}</p>
                                <p>Audio Volume: {item.audio_volume}</p>
                                <p>Pet Name: {item.pet_name}</p>
                                <p>Pet Prefix: {item.pet_prefix}</p>
                                <p>Pet Suffix: {item.pet_suffix}</p>
                                <p>Pet Ability: {item.pet_ability}</p>
                                <p>Seed Base Sprite: {item.seed_base_sprite}</p>
                                <p>
                                  Seed Overlay Sprite:{" "}
                                  {item.seed_overlay_sprite}
                                </p>
                                <p>Tree Base Sprite: {item.tree_base_sprite}</p>
                                <p>
                                  Tree Overlay Sprite:{" "}
                                  {item.tree_overlay_sprite}
                                </p>
                                <p>Base Color: {item.base_color}</p>
                                <p>Overlay Color: {item.overlay_color}</p>
                                <p>Ingredient: {item.ingredient}</p>
                                <p>Grow Time: {item.grow_time}</p>
                                <p>Is Rayman: {item.is_rayman}</p>
                                <p>Extra Options: {item.extra_options}</p>
                                <p>Texture Path 2: {item.texture_path_2}</p>
                                <p>Extra Option 2: {item.extra_option2}</p>
                                <p>Punch Option: {item.punch_option}</p>
                              </div>
                            </div>
                          </DialogContent>
                        </Dialog>
                      ))}
                  </>
                )}
              </div>
            </>
          )}
        </TabsContent>
      </Tabs>
    </div>
  );
}

export default App;
