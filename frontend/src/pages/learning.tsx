import { Switch } from "@/components/ui/switch";
import { Button } from "@/components/ui/button";
import { BookOpen, Trash2 } from "lucide-react";
import { useEffect, useState } from "react";
import { toast } from "sonner";
import { invoke } from '@tauri-apps/api/core';
import {
    AlertDialog,
    AlertDialogAction,
    AlertDialogCancel,
    AlertDialogContent,
    AlertDialogDescription,
    AlertDialogFooter,
    AlertDialogHeader,
    AlertDialogTitle,
    AlertDialogTrigger,
} from "@/components/ui/alert-dialog";

export const Learning = () => {
    const [value, setValue] = useState({
        enabled: true,
    });

    // Load config on component mount
    useEffect(() => {
        invoke<any>("get_config")
            .then((data) => {
                const learning = data.learning;
                setValue({
                    enabled: learning?.enabled ?? true,
                });
            })
            .catch(() => {
                // Keep default values if config fetch fails
            });
    }, []);

    const updateConfig = async (updater: (config: any) => void) => {
        try {
            const data = await invoke<any>("get_config");
            updater(data);
            await invoke("update_config", { newConfig: data });
            return data;
        } catch (error) {
            toast("設定の更新に失敗しました");
            return null;
        }
    };

    const handleLearningChange = async () => {
        const data = await updateConfig((data) => {
            if (!data.learning) {
                data.learning = { enabled: true };
            }
            data.learning.enabled = !value.enabled;
        });

        if (data) {
            setValue((prev) => ({ ...prev, enabled: data.learning.enabled }));
            toast(data.learning.enabled ? "学習機能を有効にしました" : "学習機能を無効にしました");
        }
    };

    const handleResetLearning = async () => {
        try {
            await invoke("reset_learning");
            toast("学習データをリセットしました", {
                description: "変更を完全に適用するには、PCを再起動してください",
                duration: 10000,
            });
        } catch (error) {
            toast("学習データのリセットに失敗しました");
        }
    };

    return (
        <div className="space-y-8">
            <section className="space-y-2">
                <h1 className="text-sm font-bold text-foreground">学習</h1>
                <div className="flex items-center space-x-4 rounded-md border p-4">
                    <BookOpen />
                    <div className="flex-1 space-y-1">
                        <p className="text-sm font-medium leading-none">
                            学習機能を有効化
                        </p>
                        <p className="text-xs text-muted-foreground">
                            変換履歴を学習して、よく使う候補を優先的に表示します
                        </p>
                    </div>
                    <Switch checked={value.enabled} onCheckedChange={handleLearningChange} />
                </div>
                <div className="flex items-center space-x-4 rounded-md border p-4">
                    <Trash2 />
                    <div className="flex-1 space-y-1">
                        <p className="text-sm font-medium leading-none">
                            学習データのリセット
                        </p>
                        <p className="text-xs text-muted-foreground">
                            蓄積された学習データを削除します
                        </p>
                    </div>
                    <AlertDialog>
                        <AlertDialogTrigger asChild>
                            <Button variant="destructive" disabled={!value.enabled}>
                                リセット
                            </Button>
                        </AlertDialogTrigger>
                        <AlertDialogContent>
                            <AlertDialogHeader>
                                <AlertDialogTitle>学習データをリセットしますか？</AlertDialogTitle>
                                <AlertDialogDescription>
                                    この操作は取り消せません。蓄積されたすべての学習データが削除されます。
                                </AlertDialogDescription>
                            </AlertDialogHeader>
                            <AlertDialogFooter>
                                <AlertDialogCancel>キャンセル</AlertDialogCancel>
                                <AlertDialogAction onClick={handleResetLearning}>
                                    リセット
                                </AlertDialogAction>
                            </AlertDialogFooter>
                        </AlertDialogContent>
                    </AlertDialog>
                </div>
            </section>
        </div>
    );
};

