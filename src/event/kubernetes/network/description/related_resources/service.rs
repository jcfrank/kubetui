pub mod filter_by_names {
    use k8s_openapi::api::core::v1::Service;

    use crate::event::kubernetes::{
        client::KubeClientRequest,
        network::description::related_resources::{fetch::FetchClient, RelatedResources},
    };

    use super::*;

    pub struct RelatedService<'a, C: KubeClientRequest> {
        client: FetchClient<'a, C>,
        names: Vec<String>,
    }

    impl<'a, C: KubeClientRequest> RelatedService<'a, C> {
        pub fn new(client: &'a C, namespace: &'a str, names: Vec<String>) -> Self {
            Self {
                client: FetchClient::new(client, namespace),
                names,
            }
        }
    }

    #[async_trait::async_trait]
    impl<'a, C: KubeClientRequest> RelatedResources<C> for RelatedService<'a, C> {
        type Item = Vec<String>;
        type Filtered = Service;

        fn client(&self) -> &FetchClient<C> {
            &self.client
        }

        fn item(&self) -> &Self::Item {
            &self.names
        }
    }

    mod filter {
        use crate::event::kubernetes::network::description::related_resources::Filter;
        use k8s_openapi::List;
        use kube::ResourceExt;

        use super::*;

        trait FilterByNames: Filter {}

        impl Filter for List<Service> {
            type Item = Vec<String>;

            type Filtered = Service;

            fn filter_by_item(&self, arg: &Self::Item) -> Option<List<Self::Filtered>>
            where
                Self::Filtered: k8s_openapi::ListableResource,
            {
                let ret: Vec<Service> = self
                    .items
                    .iter()
                    .filter(|svc| arg.iter().any(|name| &svc.name() == name))
                    .cloned()
                    .collect();

                if !ret.is_empty() {
                    Some(List {
                        items: ret,
                        ..Default::default()
                    })
                } else {
                    None
                }
            }
        }

        #[cfg(test)]
        mod tests {
            use indoc::indoc;

            use super::*;

            fn services() -> List<Service> {
                let yaml = indoc! {
                    "
                    items:
                      - metadata:
                          name: service-1
                      - metadata:
                          name: service-2
                      - metadata:
                          name: service-3
                    "
                };

                serde_yaml::from_str(&yaml).unwrap()
            }

            #[test]
            fn namesに一致するserviceのリストを返す() {
                let arg = vec!["service-1".into(), "service-2".into()];

                let list = services();

                let actual = list.filter_by_item(&arg);

                let expected = serde_yaml::from_str(indoc! {
                    "
                    items:
                      - metadata:
                          name: service-1
                      - metadata:
                          name: service-2
                    "
                })
                .unwrap();

                assert_eq!(actual, Some(expected))
            }

            #[test]
            fn namesに一致するserviceがないときnoneを返す() {
                let arg = vec!["hoge".into()];

                let list = services();

                let actual = list.filter_by_item(&arg);

                assert_eq!(actual.is_none(), true)
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        use anyhow::bail;
        use indoc::indoc;
        use k8s_openapi::{api::core::v1::Service, List};
        use mockall::predicate::eq;
        use pretty_assertions::assert_eq;
        use serde_yaml::Value;

        use crate::{event::kubernetes::client::mock::MockTestKubeClient, mock_expect};

        fn services() -> List<Service> {
            let yaml = indoc! {
                "
                items:
                  - metadata:
                      name: service-1
                  - metadata:
                      name: service-2
                  - metadata:
                      name: service-3
                "
            };

            serde_yaml::from_str(&yaml).unwrap()
        }

        #[tokio::test]
        async fn nameリストに含まれるservice名のvalueを返す() {
            let mut client = MockTestKubeClient::new();

            mock_expect!(
                client,
                request,
                List<Service>,
                eq("/api/v1/namespaces/default/services"),
                Ok(services())
            );

            let client = RelatedService::new(
                &client,
                "default",
                vec!["service-1".into(), "service-3".into()],
            );

            let result = client.related_resources().await.unwrap().unwrap();

            let expected = Value::from(vec!["service-1", "service-3"]);

            assert_eq!(result, expected);
        }

        #[tokio::test]
        async fn nameリストに含まれるserviceがないときnoneを返す() {
            let mut client = MockTestKubeClient::new();

            mock_expect!(
                client,
                request,
                List<Service>,
                eq("/api/v1/namespaces/default/services"),
                Ok(services())
            );

            let client = RelatedService::new(&client, "default", vec!["hoge".into()]);

            let result = client.related_resources().await.unwrap();

            assert_eq!(result.is_none(), true);
        }

        #[tokio::test]
        async fn エラーがでたときerrを返す() {
            let mut client = MockTestKubeClient::new();

            mock_expect!(
                client,
                request,
                List<Service>,
                eq("/api/v1/namespaces/default/services"),
                bail!("error")
            );

            let client = RelatedService::new(&client, "default", vec!["service-1".into()]);

            let result = client.related_resources().await;

            assert_eq!(result.is_err(), true);
        }
    }
}

pub mod filter_by_selector {
    use super::*;

    mod filter {
        use super::*;

        #[test]
        fn labelsにselectorの値を含むときそのserviceのリストを返す() {}

        #[test]
        fn labelsにselectorの値を含まないときnoneを返す() {}
    }

    #[cfg(test)]
    mod tests {
        use anyhow::bail;
        use indoc::indoc;
        use k8s_openapi::{api::core::v1::Service, List};
        use mockall::predicate::eq;
        use serde_yaml::Value;

        use crate::{event::kubernetes::client::mock::MockTestKubeClient, mock_expect};

        use super::*;

        fn services() -> List<Service> {
            let yaml = indoc! {
                "
                items:
                  - metadata:
                      name: service-1
                    spec:
                      selector:
                        app: pod-1
                        version: v1
                  - metadata:
                      name: service-2
                    spec:
                      selector:
                        app: pod-2
                        version: v1
                  - metadata:
                      name: service-3
                    spec:
                      selector:
                        app: pod-3
                        version: v2
                "
            };

            serde_yaml::from_str(&yaml).unwrap()
        }

        #[tokio::test]
        async fn labelsリストに含まれるservice名のvalueを返す() {
            let mut client = MockTestKubeClient::new();

            mock_expect!(
                client,
                request,
                List<Service>,
                eq("/api/v1/namespaces/default/services"),
                Ok(services())
            );

            let client = RelatedService::new(
                &client,
                "default",
                BTreeMap::from([("version".to_string(), "v1".to_string())]),
            );

            let result = client.related_resources().await.unwrap().unwrap();

            let expected = Value::from(vec!["service-1", "service-2"]);

            assert_eq!(result, expected);
        }

        #[tokio::test]
        async fn labelsリストに含まれるserviceがないときnoneを返す() {
            let mut client = MockTestKubeClient::new();

            mock_expect!(
                client,
                request,
                List<Service>,
                eq("/api/v1/namespaces/default/services"),
                Ok(services())
            );

            let client = RelatedService::new(
                &client,
                "default",
                BTreeMap::from([("foo".to_string(), "bar".to_string())]),
            );

            let result = client.related_resources().await.unwrap();

            assert_eq!(result.is_none(), true);
        }

        #[tokio::test]
        async fn エラーがでたときerrを返す() {
            let mut client = MockTestKubeClient::new();

            mock_expect!(
                client,
                request,
                List<Service>,
                eq("/api/v1/namespaces/default/services"),
                bail!("error")
            );

            let client = RelatedService::new(
                &client,
                "default",
                BTreeMap::from([("version".to_string(), "v1".to_string())]),
            );

            let result = client.related_resources().await;

            assert_eq!(result.is_err(), true);
        }
    }
}
