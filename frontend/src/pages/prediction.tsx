import { Switch } from "@/components/ui/switch";
import { Lightbulb } from "lucide-react";
import { useEffect, useState } from "react";
import { toast } from "sonner";
import { invoke } from '@tauri-apps/api/core';

export const Prediction = () => {
    const [value, setValue] = useState({
        enabled: true,
    });

    // Load config on component mount
    useEffect(() => {
        invoke<any>("get_config")
            .then((data) => {
                const prediction = data.prediction;
                setValue({
                    enabled: prediction?.enabled ?? true,
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

    const handlePredictionChange = async () => {
        const data = await updateConfig((data) => {
            if (!data.prediction) {
                data.prediction = { enabled: true };
            }
            data.prediction.enabled = !value.enabled;
        });

        if (data) {
            setValue((prev) => ({ ...prev, enabled: data.prediction.enabled }));
            toast(data.prediction.enabled ? "予測変換を有効にしました" : "予測変換を無効にしました", {
                description: "変更を完全に適用するには、PCを再起動してください",
                duration: 10000,
            });
        }
    };

    return (
        <div className="space-y-8">
            <section className="space-y-2">
                <h1 className="text-sm font-bold text-foreground">予測変換</h1>
                <div className="flex items-center space-x-4 rounded-md border p-4">
                    <Lightbulb />
                    <div className="flex-1 space-y-1">
                        <p className="text-sm font-medium leading-none">
                            予測変換を有効化
                        </p>
                        <p className="text-xs text-muted-foreground">
                            入力中に次に入力される可能性の高い単語を予測して候補に表示します
                        </p>
                    </div>
                    <Switch checked={value.enabled} onCheckedChange={handlePredictionChange} />
                </div>
                <div className="rounded-md border p-4 bg-muted/50">
                    <p className="text-xs text-muted-foreground">
                        💡 予測変換は、入力中のひらがなに続く候補を表示します。変換履歴や学習データと組み合わせることで、より正確な予測が可能になります。
                    </p>
                </div>
            </section>
        </div>
    );
};

