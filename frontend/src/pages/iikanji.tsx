import { Switch } from "@/components/ui/switch";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Sparkles, Key, AlertTriangle, CheckCircle } from "lucide-react";
import { useEffect, useState } from "react";
import { toast } from "sonner";
import { invoke } from '@tauri-apps/api/core';

interface IikanjiConfig {
    enabled: boolean;
    provider: string;
    api_key: string;
    model: string;
    max_tokens: number;
    temperature: number;
}

const defaultConfig: IikanjiConfig = {
    enabled: false,
    provider: "openai",
    api_key: "",
    model: "gpt-4o-mini",
    max_tokens: 256,
    temperature: 0.7,
};

const keywords = [
    { keyword: "えいご / エイゴ", description: "直前の日本語を英語に翻訳" },
    { keyword: "にほんご / ニホンゴ", description: "直前の英語を日本語に翻訳" },
    { keyword: "えもじ / エモジ", description: "文脈に適した絵文字を推薦" },
    { keyword: "いいかえ / イイカエ", description: "直前の文を別の表現に言い換え" },
    { keyword: "けいご / ケイゴ", description: "直前の文を敬語に変換" },
    { keyword: "ためご / タメゴ", description: "直前の文をカジュアルに変換" },
    { keyword: "こうせい / コウセイ", description: "文法・誤字脱字を校正" },
];

const models = [
    { value: "gpt-4o-mini", label: "GPT-4o Mini (推奨)" },
    { value: "gpt-4o", label: "GPT-4o" },
    { value: "gpt-4-turbo", label: "GPT-4 Turbo" },
    { value: "gpt-3.5-turbo", label: "GPT-3.5 Turbo" },
];

export const Iikanji = () => {
    const [config, setConfig] = useState<IikanjiConfig>(defaultConfig);
    const [apiKeyVisible, setApiKeyVisible] = useState(false);
    const [testStatus, setTestStatus] = useState<"idle" | "testing" | "success" | "error">("idle");

    useEffect(() => {
        invoke<any>("get_config")
            .then((data) => {
                if (data.iikanji) {
                    setConfig({
                        enabled: data.iikanji.enabled ?? false,
                        provider: data.iikanji.provider ?? "openai",
                        api_key: data.iikanji.api_key ?? "",
                        model: data.iikanji.model ?? "gpt-4o-mini",
                        max_tokens: data.iikanji.max_tokens ?? 256,
                        temperature: data.iikanji.temperature ?? 0.7,
                    });
                }
            })
            .catch(() => {
                // Keep default values if config fetch fails
            });
    }, []);

    const updateConfig = async (newConfig: Partial<IikanjiConfig>) => {
        try {
            const data = await invoke<any>("get_config");
            if (!data.iikanji) {
                data.iikanji = { ...defaultConfig };
            }
            Object.assign(data.iikanji, newConfig);
            await invoke("update_config", { newConfig: data });
            setConfig((prev) => ({ ...prev, ...newConfig }));
            return true;
        } catch (error) {
            toast.error("設定の更新に失敗しました");
            return false;
        }
    };

    const handleEnabledChange = async () => {
        const newEnabled = !config.enabled;
        const success = await updateConfig({ enabled: newEnabled });
        if (success) {
            toast(newEnabled ? "いい感じ変換を有効にしました" : "いい感じ変換を無効にしました", {
                description: "変更を完全に適用するには、PCを再起動してください",
                duration: 10000,
            });
        }
    };

    const handleApiKeyChange = async (value: string) => {
        setConfig((prev) => ({ ...prev, api_key: value }));
    };

    const handleApiKeySave = async () => {
        const success = await updateConfig({ api_key: config.api_key });
        if (success) {
            toast.success("APIキーを保存しました");
        }
    };

    const handleModelChange = async (value: string) => {
        const success = await updateConfig({ model: value });
        if (success) {
            toast.success(`モデルを ${value} に変更しました`);
        }
    };

    const testApiKey = async () => {
        if (!config.api_key) {
            toast.error("APIキーを入力してください");
            return;
        }

        setTestStatus("testing");
        
        try {
            // Simple test request to OpenAI API
            const response = await fetch("https://api.openai.com/v1/models", {
                headers: {
                    "Authorization": `Bearer ${config.api_key}`,
                },
            });

            if (response.ok) {
                setTestStatus("success");
                toast.success("APIキーは有効です");
            } else if (response.status === 401) {
                setTestStatus("error");
                toast.error("APIキーが無効です");
            } else {
                setTestStatus("error");
                toast.error(`エラー: ${response.status}`);
            }
        } catch (error) {
            setTestStatus("error");
            toast.error("接続エラー");
        }

        setTimeout(() => setTestStatus("idle"), 3000);
    };

    return (
        <div className="space-y-8">
            <section className="space-y-2">
                <h1 className="text-sm font-bold text-foreground">いい感じ変換</h1>
                <div className="flex items-center space-x-4 rounded-md border p-4">
                    <Sparkles className="h-5 w-5" />
                    <div className="flex-1 space-y-1">
                        <p className="text-sm font-medium leading-none">
                            いい感じ変換を有効化
                        </p>
                        <p className="text-xs text-muted-foreground">
                            特殊キーワードを使ってLLMによる翻訳、言い換え、絵文字推薦などを行います
                        </p>
                    </div>
                    <Switch checked={config.enabled} onCheckedChange={handleEnabledChange} />
                </div>
            </section>

            <section className="space-y-2">
                <h2 className="text-sm font-bold text-foreground">LLM設定</h2>
                
                <div className="space-y-4 rounded-md border p-4">
                    <div className="space-y-2">
                        <label className="text-sm font-medium">APIキー</label>
                        <div className="flex space-x-2">
                            <div className="relative flex-1">
                                <Key className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                                <Input
                                    type={apiKeyVisible ? "text" : "password"}
                                    placeholder="sk-..."
                                    value={config.api_key}
                                    onChange={(e) => handleApiKeyChange(e.target.value)}
                                    className="pl-10"
                                />
                            </div>
                            <Button
                                variant="outline"
                                size="sm"
                                onClick={() => setApiKeyVisible(!apiKeyVisible)}
                            >
                                {apiKeyVisible ? "隠す" : "表示"}
                            </Button>
                            <Button
                                variant="outline"
                                size="sm"
                                onClick={handleApiKeySave}
                            >
                                保存
                            </Button>
                            <Button
                                variant="outline"
                                size="sm"
                                onClick={testApiKey}
                                disabled={testStatus === "testing"}
                            >
                                {testStatus === "testing" ? "テスト中..." : 
                                 testStatus === "success" ? <CheckCircle className="h-4 w-4 text-green-500" /> :
                                 testStatus === "error" ? <AlertTriangle className="h-4 w-4 text-red-500" /> :
                                 "テスト"}
                            </Button>
                        </div>
                        <p className="text-xs text-muted-foreground">
                            OpenAIのAPIキーを入力してください。
                            <a 
                                href="https://platform.openai.com/api-keys" 
                                target="_blank" 
                                rel="noopener noreferrer"
                                className="text-primary hover:underline ml-1"
                            >
                                APIキーを取得 →
                            </a>
                        </p>
                    </div>

                    <div className="space-y-2">
                        <label className="text-sm font-medium">モデル</label>
                        <Select value={config.model} onValueChange={handleModelChange}>
                            <SelectTrigger>
                                <SelectValue placeholder="モデルを選択" />
                            </SelectTrigger>
                            <SelectContent>
                                {models.map((model) => (
                                    <SelectItem key={model.value} value={model.value}>
                                        {model.label}
                                    </SelectItem>
                                ))}
                            </SelectContent>
                        </Select>
                        <p className="text-xs text-muted-foreground">
                            GPT-4o Miniは高速で低コスト、GPT-4oは高精度です
                        </p>
                    </div>
                </div>
            </section>

            <section className="space-y-2">
                <h2 className="text-sm font-bold text-foreground">使用可能なキーワード</h2>
                <div className="rounded-md border">
                    <table className="w-full">
                        <thead>
                            <tr className="border-b bg-muted/50">
                                <th className="p-3 text-left text-xs font-medium">キーワード</th>
                                <th className="p-3 text-left text-xs font-medium">機能</th>
                            </tr>
                        </thead>
                        <tbody>
                            {keywords.map((item, index) => (
                                <tr key={index} className="border-b last:border-0">
                                    <td className="p-3 text-sm font-mono">{item.keyword}</td>
                                    <td className="p-3 text-sm text-muted-foreground">{item.description}</td>
                                </tr>
                            ))}
                        </tbody>
                    </table>
                </div>
            </section>

            <section className="space-y-2">
                <div className="rounded-md border border-yellow-500/50 bg-yellow-500/10 p-4">
                    <div className="flex items-start space-x-3">
                        <AlertTriangle className="h-5 w-5 text-yellow-500 mt-0.5" />
                        <div className="space-y-1">
                            <p className="text-sm font-medium text-yellow-500">プライバシーに関する注意</p>
                            <p className="text-xs text-muted-foreground">
                                いい感じ変換を使用すると、入力内容がOpenAIのサーバーに送信されます。
                                機密情報や個人情報の入力にはご注意ください。
                            </p>
                        </div>
                    </div>
                </div>
            </section>
        </div>
    );
};

