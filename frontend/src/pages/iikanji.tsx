import { Switch } from "@/components/ui/switch";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Sparkles, Key, AlertTriangle, CheckCircle, Cpu, Cloud, Shield } from "lucide-react";
import { useEffect, useState } from "react";
import { toast } from "sonner";
import { invoke } from '@tauri-apps/api/core';

interface OpenAIConfig {
    api_key: string;
    model: string;
    max_tokens: number;
    temperature: number;
}

interface IikanjiConfig {
    enabled: boolean;
    provider: string;
    openai: OpenAIConfig;
}

const defaultConfig: IikanjiConfig = {
    enabled: false,
    provider: "zenzai",
    openai: {
        api_key: "",
        model: "gpt-5-mini",  // GPT-5シリーズのコスト効率版
        max_tokens: 256,
        temperature: 0.7,
    },
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

// OpenAIモデルリスト（最新順）
// OpenAIモデルリスト（2025年11月時点）
const openaiModels = [
    // GPT-5シリーズ（最新・推奨）
    { value: "gpt-5.1", label: "GPT-5.1 (最新・最高性能)", group: "GPT-5" },
    { value: "gpt-5", label: "GPT-5 (高性能・安定)", group: "GPT-5" },
    { value: "gpt-5-mini", label: "GPT-5 Mini (高速・低コスト)", group: "GPT-5" },
    { value: "gpt-5-nano", label: "GPT-5 Nano (最速・最低コスト)", group: "GPT-5" },
    // GPT-4.1（非推論モデル）
    { value: "gpt-4.1", label: "GPT-4.1 (非推論・高速)", group: "GPT-4.1" },
    // GPT-4o系（レガシー）
    { value: "gpt-4o", label: "GPT-4o (レガシー)", group: "レガシー" },
    { value: "gpt-4o-mini", label: "GPT-4o Mini (レガシー・低コスト)", group: "レガシー" },
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
                        provider: data.iikanji.provider ?? "zenzai",
                        openai: {
                            api_key: data.iikanji.openai?.api_key ?? "",
                            model: data.iikanji.openai?.model ?? "gpt-4o-mini",
                            max_tokens: data.iikanji.openai?.max_tokens ?? 256,
                            temperature: data.iikanji.openai?.temperature ?? 0.7,
                        },
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
            // Deep merge for openai config
            if (newConfig.openai) {
                data.iikanji.openai = { ...data.iikanji.openai, ...newConfig.openai };
            }
            if (newConfig.enabled !== undefined) {
                data.iikanji.enabled = newConfig.enabled;
            }
            if (newConfig.provider !== undefined) {
                data.iikanji.provider = newConfig.provider;
            }
            await invoke("update_config", { newConfig: data });
            setConfig((prev) => ({
                ...prev,
                ...newConfig,
                openai: newConfig.openai ? { ...prev.openai, ...newConfig.openai } : prev.openai,
            }));
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

    const handleProviderChange = async (value: string) => {
        const success = await updateConfig({ provider: value });
        if (success) {
            toast.success(value === "zenzai" 
                ? "Zenzai（ローカル）に切り替えました" 
                : "OpenAI（クラウド）に切り替えました");
        }
    };

    const handleApiKeyChange = async (value: string) => {
        setConfig((prev) => ({ 
            ...prev, 
            openai: { ...prev.openai, api_key: value } 
        }));
    };

    const handleApiKeySave = async () => {
        const success = await updateConfig({ openai: { ...config.openai } });
        if (success) {
            toast.success("APIキーを保存しました");
        }
    };

    const handleModelChange = async (value: string) => {
        const success = await updateConfig({ openai: { ...config.openai, model: value } });
        if (success) {
            toast.success(`モデルを ${value} に変更しました`);
        }
    };

    const testApiKey = async () => {
        if (!config.openai.api_key) {
            toast.error("APIキーを入力してください");
            return;
        }

        setTestStatus("testing");
        
        try {
            const response = await fetch("https://api.openai.com/v1/models", {
                headers: {
                    "Authorization": `Bearer ${config.openai.api_key}`,
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
                <h2 className="text-sm font-bold text-foreground">プロバイダー選択</h2>
                
                <div className="grid grid-cols-2 gap-4">
                    {/* Zenzai（ローカル） */}
                    <div 
                        className={`rounded-md border p-4 cursor-pointer transition-all ${
                            config.provider === "zenzai" 
                                ? "border-primary bg-primary/5" 
                                : "hover:border-muted-foreground/50"
                        }`}
                        onClick={() => handleProviderChange("zenzai")}
                    >
                        <div className="flex items-center space-x-3 mb-2">
                            <Cpu className={`h-5 w-5 ${config.provider === "zenzai" ? "text-primary" : ""}`} />
                            <span className="font-medium">Zenzai（ローカル）</span>
                        </div>
                        <div className="space-y-1 text-xs text-muted-foreground">
                            <div className="flex items-center space-x-1">
                                <Shield className="h-3 w-3 text-green-500" />
                                <span>完全オフライン・プライバシー保護</span>
                            </div>
                            <p>azooKeyのニューラル変換エンジンを使用</p>
                            <p className="text-green-600">無料・追加設定不要</p>
                        </div>
                    </div>

                    {/* OpenAI（クラウド） */}
                    <div 
                        className={`rounded-md border p-4 cursor-pointer transition-all ${
                            config.provider === "openai" 
                                ? "border-primary bg-primary/5" 
                                : "hover:border-muted-foreground/50"
                        }`}
                        onClick={() => handleProviderChange("openai")}
                    >
                        <div className="flex items-center space-x-3 mb-2">
                            <Cloud className={`h-5 w-5 ${config.provider === "openai" ? "text-primary" : ""}`} />
                            <span className="font-medium">OpenAI（クラウド）</span>
                        </div>
                        <div className="space-y-1 text-xs text-muted-foreground">
                            <div className="flex items-center space-x-1">
                                <AlertTriangle className="h-3 w-3 text-yellow-500" />
                                <span>インターネット接続が必要</span>
                            </div>
                            <p>GPT-4o等の高精度モデルを使用</p>
                            <p className="text-yellow-600">APIキー・利用料金が必要</p>
                        </div>
                    </div>
                </div>
            </section>

            {/* OpenAI設定（OpenAI選択時のみ表示） */}
            {config.provider === "openai" && (
                <section className="space-y-2">
                    <h2 className="text-sm font-bold text-foreground">OpenAI設定</h2>
                    
                    <div className="space-y-4 rounded-md border p-4">
                        <div className="space-y-2">
                            <label className="text-sm font-medium">APIキー</label>
                            <div className="flex space-x-2">
                                <div className="relative flex-1">
                                    <Key className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                                    <Input
                                        type={apiKeyVisible ? "text" : "password"}
                                        placeholder="sk-..."
                                        value={config.openai.api_key}
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
                            <Select value={config.openai.model} onValueChange={handleModelChange}>
                                <SelectTrigger>
                                    <SelectValue placeholder="モデルを選択" />
                                </SelectTrigger>
                                <SelectContent>
                                    {openaiModels.map((model) => (
                                        <SelectItem key={model.value} value={model.value}>
                                            {model.label}
                                        </SelectItem>
                                    ))}
                                </SelectContent>
                            </Select>
                            <p className="text-xs text-muted-foreground">
                                GPT-4o Miniは高速で低コスト、GPT-4oは高精度です。o1/o3系は推論に特化しています。
                            </p>
                        </div>
                    </div>
                </section>
            )}

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

            {/* プライバシー警告（OpenAI選択時のみ） */}
            {config.provider === "openai" && (
                <section className="space-y-2">
                    <div className="rounded-md border border-yellow-500/50 bg-yellow-500/10 p-4">
                        <div className="flex items-start space-x-3">
                            <AlertTriangle className="h-5 w-5 text-yellow-500 mt-0.5" />
                            <div className="space-y-1">
                                <p className="text-sm font-medium text-yellow-500">プライバシーに関する注意</p>
                                <p className="text-xs text-muted-foreground">
                                    OpenAIプロバイダーを使用すると、入力内容がOpenAIのサーバーに送信されます。
                                    機密情報や個人情報の入力にはご注意ください。
                                    プライバシーを重視する場合は、Zenzai（ローカル）をお使いください。
                                </p>
                            </div>
                        </div>
                    </div>
                </section>
            )}

            {/* Zenzai使用時の情報 */}
            {config.provider === "zenzai" && (
                <section className="space-y-2">
                    <div className="rounded-md border border-green-500/50 bg-green-500/10 p-4">
                        <div className="flex items-start space-x-3">
                            <Shield className="h-5 w-5 text-green-500 mt-0.5" />
                            <div className="space-y-1">
                                <p className="text-sm font-medium text-green-600">プライバシー保護モード</p>
                                <p className="text-xs text-muted-foreground">
                                    Zenzaiはローカルで動作するため、入力内容は外部に送信されません。
                                    機密情報も安全に処理できます。
                                </p>
                                <p className="text-xs text-muted-foreground mt-2">
                                    <strong>注意:</strong> Zenzaiでいい感じ変換を使用するには、設定の「Zenzai」でZenzaiを有効にする必要があります。
                                </p>
                            </div>
                        </div>
                    </div>
                </section>
            )}
        </div>
    );
};
