import { Button } from "@/components/ui/button";
import { createFileRoute } from "@tanstack/react-router";
import { useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { desktopDir } from "@tauri-apps/api/path";
import { toast } from "sonner";
import { commands } from "@/bindings";

export const Route = createFileRoute("/")({
  component: Index,
});

function Index() {
  const [cropDir, setCropDir] = useState<string>();
  const [imageDir, setImageDir] = useState<string>();

  async function onClick(type: "crop" | "image") {
    const selectedDir = await open({
      multiple: false,
      directory: true,
      // defaultPath: await desktopDir(),
    });
    if (selectedDir == null) {
      toast.error("No directory selected");
    } else if (type === "crop") {
      setCropDir(selectedDir);
    } else {
      setImageDir(selectedDir);
    }
  }

  async function cropOnClick() {
    if (cropDir === undefined || imageDir === undefined) {
      toast.error("Either the Crop Folder or the Image Folder is not selected");
    } else {
      const res = await commands.crop(cropDir, imageDir);
      if (res.status === "ok") {
        if (!res.data) {
          toast.error("Following file errored", res.data);
        }
      } else {
        toast.error(res.error);
      }
    }
  }

  return (
    <div className="flex h-full w-full flex-col">
      <div className="flex flex-row items-center justify-center gap-2">
        <div className="flex flex-col gap-2">
          <Button onClick={() => onClick("crop")}>SELECT CROP FOLDER</Button>
          {cropDir && <span>{cropDir}</span>}
        </div>

        <div className="flex flex-col gap-2">
          <Button onClick={() => onClick("image")}>SELECT IMAGE FOLDER</Button>
          {cropDir && <span>{imageDir}</span>}
        </div>
      </div>

      <div className="flex h-full w-full items-center justify-center gap-2">
        <Button onClick={cropOnClick} className="h-1/3 w-1/3">CROP TUDO BICHO!</Button>
      </div>
    </div>
  );
}
